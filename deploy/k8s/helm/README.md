# R-Nacos Helm Chart

这是一个用于在 Kubernetes 集群中部署 R-Nacos（Rust 版本的 Nacos）的 Helm Chart。

## 简介

R-Nacos 是用 Rust 语言重写的 Nacos，提供了服务发现、配置管理和服务管理平台。此 Helm Chart 支持集群模式部署，使用 Raft 协议确保高可用性。

## 先决条件

- Kubernetes 1.16+
- Helm 3.0+
- 动态存储卷配置（推荐）

## 安装

### 添加 Helm 仓库（如果适用）

```bash
# 如果有公共仓库
helm repo add rnacos <repository-url>
helm repo update
```

### 从本地安装

```bash
# 克隆仓库
git clone https://github.com/nacos-group/r-nacos.git
cd r-nacos/deploy/k8s/helm

# 安装 Chart
helm install my-rnacos . -n rnacos --create-namespace
```

### 自定义安装

```bash
# 使用自定义 values 文件
helm install my-rnacos . -n rnacos --create-namespace -f my-values.yaml

# 或者使用命令行参数
helm install my-rnacos . -n rnacos --create-namespace \
  --set replicaCount=3 \
  --set persistence.size=50Gi \
  --set ingress.enabled=true
```

## 配置

以下表格列出了可配置的参数和默认值：

### 基础配置

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `replicaCount` | 副本数量 | `3` |
| `image.repository` | 镜像仓库 | `qingpan/rnacos` |
| `image.tag` | 镜像标签 | `latest` |
| `image.pullPolicy` | 镜像拉取策略 | `Always` |

### R-Nacos 配置 (已废弃)

> ⚠️ **废弃警告**: `rancosConfig` 已废弃，请使用 `env` 配置方式

| 参数 | 描述 | 默认值 | 新的替代方式 |
|------|------|--------|-------------|
| `rancosConfig.rustLog` | 日志级别 | `info` | `env.RUST_LOG` |
| `rancosConfig.httpWorkers` | HTTP 工作线程数 | `8` | `env.RNACOS_HTTP_WORKERS` |
| `rancosConfig.consoleLoginOneHourLimit` | 控制台登录限制 | `5` | `env.RNACOS_CONSOLE_LOGIN_ONE_HOUR_LIMIT` |

### 环境变量配置 (推荐)

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `env.RUST_LOG` | 日志级别 | `info` |
| `env.RNACOS_HTTP_WORKERS` | HTTP 工作线程数 | `8` |
| `env.RNACOS_CONSOLE_LOGIN_ONE_HOUR_LIMIT` | 控制台登录限制 | `5` |
| `env.RNACOS_HTTP_PORT` | HTTP 服务端口 | `8848` |
| `env.RNACOS_GRPC_PORT` | gRPC 服务端口 | `9848` |
| `env.RNACOS_HTTP_CONSOLE_PORT` | 控制台端口 | `10848` |
| `env` | 其他自定义环境变量 | `{}` |
| `envFrom` | 从 ConfigMap 或 Secret 引用环境变量 | `[]` |

### 持久化存储

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `persistence.size` | 存储大小 | `20Gi` |
| `persistence.storageClass` | 存储类 | `default` |
| `persistence.accessModes` | 访问模式 | `["ReadWriteOnce"]` |

### 服务配置

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `service.type` | 服务类型 | `ClusterIP` |
| `service.port` | 服务端口 | `10848` |

### Ingress 配置

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `ingress.enabled` | 启用 Ingress | `false` |
| `ingress.className` | Ingress 类名 | `""` |
| `ingress.hosts[0].host` | 主机名 | `rnacos.example.com` |

### 自动扩缩容

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `autoscaling.enabled` | 启用 HPA | `false` |
| `autoscaling.minReplicas` | 最小副本数 | `1` |
| `autoscaling.maxReplicas` | 最大副本数 | `100` |
| `autoscaling.targetCPUUtilizationPercentage` | CPU 使用率目标 | `80` |

## 端口说明

R-Nacos 使用以下端口：

- **8848**: HTTP 端口（客户端访问）
- **9848**: gRPC 端口（集群通信）
- **10848**: 管理端口（控制台访问、健康检查）

## 集群配置

此 Chart 默认部署 3 节点的 R-Nacos 集群：

- 使用 StatefulSet 确保稳定的网络标识
- 通过 Headless Service 进行集群内部通信
- 自动配置 Raft 集群参数
- 支持持久化存储

### 集群节点

- `rnacos-0`: 主节点，作为集群的初始节点
- `rnacos-1`, `rnacos-2`: 从节点，自动加入集群

## 健康检查

Chart 配置了完善的健康检查：

- **Readiness Probe**: HTTP GET `/health` on port 10848
- **Liveness Probe**: HTTP GET `/health` on port 10848

## 配置迁移指南

### 从 rancosConfig 迁移到 env

如果您正在使用旧的 `rancosConfig` 配置，请按以下方式迁移：

```yaml
# 旧配置 (已废弃)
rancosConfig:
  rustLog: debug
  httpWorkers: 16
  consoleLoginOneHourLimit: 10

# 新配置 (推荐)
env:
  RUST_LOG: "debug"
  RNACOS_HTTP_WORKERS: 16
  RNACOS_CONSOLE_LOGIN_ONE_HOUR_LIMIT: 10
  # 同时可以添加其他配置
  RNACOS_HTTP_PORT: 8848
  RNACOS_GRPC_PORT: 9848
  RNACOS_HTTP_CONSOLE_PORT: 10848
```

> **注意**: 当前版本仍然支持 `rancosConfig`，但建议尽快迁移到 `env` 配置方式。

## 使用示例

### 基本部署

```bash
helm install rnacos . -n rnacos --create-namespace
```

### 生产环境部署

```yaml
# production-values.yaml
replicaCount: 3

image:
  tag: "0.4.2"
  pullPolicy: IfNotPresent

# 推荐使用 env 配置
env:
  RUST_LOG: "info"
  RNACOS_HTTP_WORKERS: 16
  RNACOS_CONSOLE_LOGIN_ONE_HOUR_LIMIT: 10
  RNACOS_HTTP_PORT: 8848
  RNACOS_GRPC_PORT: 9848
  RNACOS_HTTP_CONSOLE_PORT: 10848

persistence:
  size: 100Gi
  storageClass: fast-ssd

resources:
  limits:
    cpu: 2
    memory: 4Gi
  requests:
    cpu: 1
    memory: 2Gi

ingress:
  enabled: true
  className: nginx
  hosts:
    - host: nacos.yourdomain.com
      paths:
        - path: /
          pathType: Prefix

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
```

```bash
helm install rnacos . -n rnacos --create-namespace -f production-values.yaml
```

### 环境变量配置示例

```yaml
# custom-env-values.yaml
env:
  RNACOS_CONFIG_DB_URL: "sqlite://nacos_db/config.db"
  RNACOS_NAMING_DB_URL: "sqlite://nacos_db/naming.db"
  RNACOS_CONSOLE_UI_PATH: "/ui"
  CUSTOM_DEBUG_MODE: "true"

envFrom:
  - configMapRef:
      name: rnacos-extra-config
  - secretRef:
      name: rnacos-secrets
```

```bash
helm install rnacos . -n rnacos --create-namespace -f custom-env-values.yaml
```

### 开发环境部署

```bash
helm install rnacos . -n rnacos --create-namespace \
  --set replicaCount=1 \
  --set persistence.size=10Gi \
  --set service.type=NodePort
```

## 访问应用

### 通过端口转发访问

```bash
kubectl port-forward -n rnacos svc/rnacos 8848:10848
```

然后访问 http://localhost:8848

### 通过 Ingress 访问

如果启用了 Ingress，可以通过配置的域名访问。

### 通过 NodePort 访问

如果服务类型设置为 NodePort：

```bash
kubectl get svc -n rnacos rnacos
```

## 卸载

```bash
helm uninstall rnacos -n rnacos
```

注意：这不会删除 PVC，如需删除持久化数据：

```bash
kubectl delete pvc -n rnacos -l app.kubernetes.io/instance=rnacos
```

## 故障排除

### 查看 Pod 状态

```bash
kubectl get pods -n rnacos
kubectl describe pod -n rnacos rnacos-0
```

### 查看日志

```bash
kubectl logs -n rnacos rnacos-0 -f
```

### 查看集群状态

```bash
kubectl exec -n rnacos rnacos-0 -- curl localhost:10848/health
```

### 常见问题

1. **Pod 启动失败**
   - 检查存储类是否可用
   - 确认镜像可以正常拉取
   - 查看 Pod 事件和日志

2. **集群无法形成**
   - 检查网络策略
   - 确认 Headless Service 正常工作
   - 查看各节点的环境变量配置

3. **健康检查失败**
   - 确认应用已完全启动
   - 检查 `/health` 端点是否可访问
   - 调整健康检查的超时和延迟参数

## 贡献

欢迎提交 Issue 和 Pull Request 来改进这个 Helm Chart。

## 许可证

本项目遵循与 R-Nacos 主项目相同的许可证。
