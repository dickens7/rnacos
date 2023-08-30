use std::{
    collections::{hash_map::DefaultHasher, BTreeMap, HashSet},
    hash::{Hash, Hasher},
    sync::Arc,
    time::Duration,
};

use actix::prelude::*;

use crate::{now_millis, raft::network::factory::RaftClusterRequestSender, naming::core::{NamingActor, NamingCmd}};

use super::model::NamingRouteAddr;
use super::model::NamingRouteRequest;
use super::model::SyncSenderRequest;
use super::sync_sender::ClusteSyncSender;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeStatus {
    Valid,
    Unvalid,
}

impl Default for NodeStatus {
    fn default() -> Self {
        Self::Valid
    }
}

#[derive(Default, Debug, Clone)]
pub struct ClusterNode {
    pub id: u64,
    pub index: u64,
    pub is_local: bool,
    pub addr: Arc<String>,
    pub status: NodeStatus,
}

#[derive(Default, Debug, Clone)]
pub struct ClusterInnerNode {
    pub id: u64,
    pub index: u64,
    pub is_local: bool,
    pub addr: Arc<String>,
    pub status: NodeStatus,
    pub last_active_time: u64,
    pub sync_sender: Option<Addr<ClusteSyncSender>>,
    pub client_set: HashSet<Arc<String>>,
}

impl From<ClusterInnerNode> for ClusterNode {
    fn from(value: ClusterInnerNode) -> Self {
        Self {
            id: value.id,
            index: value.index,
            is_local: value.is_local,
            addr: value.addr,
            status: value.status,
        }
    }
}

pub struct InnerNodeManage {
    local_id: u64,
    all_nodes: BTreeMap<u64, ClusterInnerNode>,
    cluster_sender: Arc<RaftClusterRequestSender>,
    naming_actor: Option<Addr<NamingActor>>,
}

impl InnerNodeManage {
    pub fn new(local_id: u64, cluster_sender: Arc<RaftClusterRequestSender>) -> Self {
        Self {
            local_id: local_id,
            cluster_sender,
            all_nodes: Default::default(),
            naming_actor: None,
        }
    }

    fn update_nodes(&mut self, nodes: Vec<(u64, Arc<String>)>) {
        let new_sets: HashSet<u64> = (&nodes).iter().map(|e| e.0.to_owned()).collect();
        let mut dels = vec![];
        for key in self.all_nodes.keys() {
            if !new_sets.contains(key) {
                dels.push(*key);
            }
        }
        for key in dels {
            self.all_nodes.remove(&key);
        }
        let now = now_millis();
        for (key, addr) in nodes {
            if let Some(node) = self.all_nodes.get_mut(&key) {
                node.addr = addr;
            } else {
                let is_local = self.local_id == key;
                let sync_sender = if is_local {
                    None
                } else {
                    Some(
                        ClusteSyncSender::new(self.local_id,key, addr.clone(), self.cluster_sender.clone())
                            .start(),
                    )
                };
                let node = ClusterInnerNode {
                    id: key,
                    index: 0,
                    is_local: is_local,
                    addr,
                    sync_sender,
                    status: NodeStatus::Valid,
                    last_active_time: now,
                    client_set: Default::default(),
                };
                self.all_nodes.insert(key, node);
            }
        }
        if !self.all_nodes.contains_key(&self.local_id) {
            let node = self.get_this_node();
            self.all_nodes.insert(self.local_id, node);
        }
        self.update_nodes_index();
    }

    fn update_nodes_index(&mut self) {
        let mut i = 0;
        for value in self.all_nodes.values_mut() {
            value.index = i.to_owned();
            i += 1;
        }
    }

    fn get_this_node(&self) -> ClusterInnerNode {
        if let Some(node) = self.all_nodes.get(&self.local_id) {
            node.to_owned()
        } else {
            ClusterInnerNode {
                id: self.local_id,
                is_local: true,
                ..Default::default()
            }
        }
    }

    fn get_all_nodes(&self) -> Vec<ClusterNode> {
        if self.all_nodes.len() == 0 {
            vec![self.get_this_node().into()]
        } else {
            self.all_nodes.values().cloned().map(|e| e.into()).collect()
        }
    }

    fn send_to_other_node(&self, req: SyncSenderRequest) {
        for node in self.all_nodes.values() {
            if node.is_local || node.status != NodeStatus::Valid {
                continue;
            }
            if let Some(sync_sender) = node.sync_sender.as_ref() {
                sync_sender.do_send(req.clone());
            }
        }
    }

    fn check_node_status(&mut self) {
        let timeout = now_millis() - 15000;
        let naming_actor = &self.naming_actor;
        for node in self.all_nodes.values_mut() {
            if !node.is_local && node.status==NodeStatus::Valid && node.last_active_time < timeout {
                node.status = NodeStatus::Unvalid;
                Self::client_unvalid_instance(naming_actor,node);
            }
        }
    }

    fn client_unvalid_instance(naming_actor:&Option<Addr<NamingActor>>,node:&mut ClusterInnerNode) {
        if let Some(naming_actor) = naming_actor.as_ref() {
            for client_id in &node.client_set {
                naming_actor.do_send(NamingCmd::RemoveClient(client_id.clone()));
            }
        }
        node.client_set.clear();
    }

    fn ping_other(&mut self) {
        let req = SyncSenderRequest(NamingRouteRequest::Ping(self.local_id));
        self.send_to_other_node(req);
    }

    fn hb(&mut self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::from_millis(3000), |act, ctx| {
            act.check_node_status();
            act.ping_other();
            act.hb(ctx);
        });
    }

    fn active_node(&mut self, node_id: u64) {
        if let Some(node) = self.all_nodes.get_mut(&node_id) {
            node.last_active_time = now_millis();
            node.status = NodeStatus::Valid;
        }
    }

    fn node_add_client(&mut self, node_id: u64,client_id: Arc<String>) {
        if let Some(node) = self.all_nodes.get_mut(&node_id) {
            node.last_active_time = now_millis();
            node.status = NodeStatus::Valid;
            if !client_id.is_empty() {
                node.client_set.insert(client_id);
            }
        }
    }
}

impl Actor for InnerNodeManage {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("InnerNodeManage started!");

        //定时检测节点的可用性
        self.hb(ctx);
    }
}

#[derive(Message, Debug)]
#[rtype(result = "anyhow::Result<NodeManageResponse>")]
pub enum NodeManageRequest {
    UpdateNodes(Vec<(u64, Arc<String>)>),
    GetThisNode,
    GetAllNodes,
    ActiveNode(u64),
    SendToOtherNodes(NamingRouteRequest),
    AddClientId(u64,Arc<String>),
    RemoveClientId(u64,Arc<String>),
    SetNamingAddr(Addr<NamingActor>),
}

pub enum NodeManageResponse {
    None,
    ThisNode(ClusterNode),
    AllNodes(Vec<ClusterNode>),
}

impl Handler<NodeManageRequest> for InnerNodeManage {
    type Result = anyhow::Result<NodeManageResponse>;

    fn handle(&mut self, msg: NodeManageRequest, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            NodeManageRequest::UpdateNodes(nodes) => {
                log::info!("InnerNodeManage UpdateNodes,size:{}", nodes.len());
                self.update_nodes(nodes);
                Ok(NodeManageResponse::None)
            }
            NodeManageRequest::GetThisNode => {
                Ok(NodeManageResponse::ThisNode(self.get_this_node().into()))
            }
            NodeManageRequest::GetAllNodes => {
                Ok(NodeManageResponse::AllNodes(self.get_all_nodes()))
            }
            NodeManageRequest::SendToOtherNodes(req) => {
                self.send_to_other_node(SyncSenderRequest(req));
                Ok(NodeManageResponse::None)
            }
            NodeManageRequest::ActiveNode(node_id) => {
                self.active_node(node_id);
                Ok(NodeManageResponse::None)
            }
            NodeManageRequest::AddClientId(node_id, client_id) => {
                self.node_add_client(node_id,client_id);
                Ok(NodeManageResponse::None)
            },
            NodeManageRequest::RemoveClientId(_, _) => todo!(),
            NodeManageRequest::SetNamingAddr(naming_actor) => {
                self.naming_actor=Some(naming_actor);
                Ok(NodeManageResponse::None)
            },
            
        }
    }
}

#[derive(Debug)]
pub struct NodeManage {
    inner_node_manage: Addr<InnerNodeManage>,
}

impl NodeManage {
    pub fn new(inner_node_manage: Addr<InnerNodeManage>) -> Self {
        Self { inner_node_manage }
    }

    pub async fn route_addr<T: Hash>(&self, v: &T) -> NamingRouteAddr {
        let mut hasher = DefaultHasher::new();
        v.hash(&mut hasher);
        let hash_value: usize = hasher.finish() as usize;
        let nodes = self.get_all_valid_nodes().await.unwrap_or_default();
        if nodes.is_empty() {
            NamingRouteAddr::Local(0)
        } else {
            let index = hash_value % nodes.len();
            let node = nodes.get(index).unwrap();
            if node.is_local {
                NamingRouteAddr::Local(index as u64)
            } else {
                NamingRouteAddr::Remote(index as u64, node.addr.clone())
            }
        }
    }

    pub async fn get_all_valid_nodes(&self) -> anyhow::Result<Vec<ClusterNode>> {
        let resp: NodeManageResponse = self
            .inner_node_manage
            .send(NodeManageRequest::GetAllNodes)
            .await??;
        match resp {
            NodeManageResponse::AllNodes(nodes) => Ok(nodes
                .into_iter()
                .filter(|e| e.status == NodeStatus::Valid)
                .collect()),
            _ => Err(anyhow::anyhow!(
                "get_all_valid_nodes error NodeManageResponse!"
            )),
        }
    }
    pub async fn get_other_valid_nodes(&self) -> anyhow::Result<Vec<ClusterNode>> {
        Ok(self
            .get_all_valid_nodes()
            .await?
            .into_iter()
            .filter(|e| !e.is_local)
            .collect())
    }

    pub fn active_node(&self, node_id: u64) {
        self.inner_node_manage
            .do_send(NodeManageRequest::ActiveNode(node_id))
    }
}
