
# r-nacos

## 简介

r-nacos是一个用rust实现的nacos服务。

r-nacos是一个轻量、 快速、稳定、高性能的服务；包含注册中心、配置中心、web管理控制台功能，支持单机、集群部署。

r-nacos设计上完全兼容最新版本nacos面向client sdk 的协议（包含1.x的http OpenApi，和2.x的grpc协议）, 支持使用nacos服务的应用平迁到 r-nacos。

r-nacos相较于java nacos来说，是一个提供相同功能，启动更快、占用系统资源更小、性能更高、运行更稳定的服务。

详细说明可以看 [r-nacos book](https://heqingpan.github.io/rnacos/index.html)

## 开发原由

一方面自己学习rust后想写个中间件实践rust网络并发编程。

另一方面自己开发应用有用到nacos，个人云服务部署一个nacos太重，本地开发测试开nacos也比较重。

本人能读java源码又能写rust，分析确认开发可行性后，决定先写一个最小功能集给自己用。

自己写完后用下来整体体验很顺畅，就想开源出来给有需要的人使用。

## 适用场景

1. 开发测试环境使用nacos，nacos服务可以换成r-nacos。启动更快，秒启动。
2. 个人资源云服务部署的 nacos，可以考虑换成r-nacos。资源占用率低: 包10M 出头，不依赖 JDK；运行时 cpu 小于0.5% ，小于5M（具体和实例有关）。
3. 使用非订制nacos服务 ，希望能提升服务性能与稳定性，可以考虑迁移到 r-nacos。

## 快速开始

### 一、 安装运行 r-nacos

【单机部署】

方式1：从 [github release](https://github.com/heqingpan/rnacos/releases) 下载对应系统的应用包，解压后即可运行。

linux 或 mac 

```shell
# 解压
tar -xvf rnacos-x86_64-apple-darwin.tar.gz
# 运行
./rnacos
```

windows 解压后直接运行 rnacos.exe 即可。

方式2:  通过docker 运行

```
#stable是最新正式版本号，也可以指定镜像版本号，如： qingpan/rnacos:v0.4.0
docker pull qingpan/rnacos:stable  
docker run --name mynacos -p 8848:8848 -p 9848:9848 -p 10848:10848 -d qingpan/rnacos:stable
```

docker 的容器运行目录是 /io，会从这个目录读写配置文件

#### docker 版本说明

应用每次打包都会同时打对应版本的docker包 ，qingpan/rnacos:$tag 。

每个版本会打两类docker包

|docker包类型|tag 格式| 示例 |说明 |
|--|--|--|--|
|gnu debian包|$version| qingpan/rnacos:v0.4.0 | docker包基于debian-slim,体积比较大(压缩包36M,解压后102M),运行性能相对较高;|
|musl alpine包|$version-alpine| qingpan/rnacos:v0.4.0-alpine | docker包基于alpine,体积比较小(压缩包11M,解压后34M),运行性能相对较低;|


如果不观注版本，可以使用最新正式版本tag: 

+ 最新的gnu正式版本: `qingpan/rnacos:stable`
+ 最新的alpine正式版本: `qingpan/rnacos:stable-alpine`

**MacOS arm系统补充说明** ：目前MacOS arm系统运行`stable`镜像失败，可以先换成`stable-alpine`镜像。等后面解决arm `stable`镜像问题后再把这个注意事项去掉。


方式3：通过 cargo 编译安装

```
# 安装
cargo install rnacos
# 运行
rnacos
```

方式4: 下载源码编译运行

```
git clone https://github.com/heqingpan/rnacos.git
cd rnacos
cargo build --release
cargo run
```

测试、试用推荐使用第1、第2种方式，直接下载就可以使用。

在linux下第1种方式默认是musl版本(性能比gnu版本差一些)，在生产服务对性能有要求的可以考虑使用第2、第3、第4种在对应环境编译gnu版本部署。

启动配置: 

| 参数KEY|内容描述|默认值|示例|开始支持的版本|
|--|--|--|--|--|
|RNACOS_HTTP_PORT|r-nacos监听http端口|8848|8848|0.1.x|
|RNACOS_GRPC_PORT|r-nacos监听grpc端口|默认是 HTTP端口+1000|9848|0.1.x|
|RNACOS_HTTP_CONSOLE_PORT|r-nacos独立控制台端口|默认是 HTTP端口+2000;设置为0可不开启独立控制台|10848|0.4.x|
|RNACOS_CONSOLE_LOGIN_ONE_HOUR_LIMIT|r-nacos控制台登录1小时失败次数限制|默认是5,一个用户连续登陆失败5次，会被锁定1个小时|5|0.4.x|
|RNACOS_HTTP_WORKERS|http工作线程数|cpu核数|8|0.1.x|
|RNACOS_CONFIG_DB_FILE|配置中心的本地数据库文件地址【0.2.x后不在使用】|config.db|config.db|0.1.x|
|RNACOS_CONFIG_DB_DIR|配置中心的本地数据库sled文件夹, 会在系统运行时自动创建|nacos_db|nacos_db|0.2.x|
|RNACOS_RAFT_NODE_ID|节点id|1|1|0.3.0|
|RNACOS_RAFT_NODE_ADDR|节点地址Ip:GrpcPort,单节点运行时每次启动都会生效；多节点集群部署时，只取加入集群时配置的值|127.0.0.1:GrpcPort|127.0.0.1:9848|0.3.0|
|RNACOS_RAFT_AUTO_INIT|是否当做主节点初始化,(只在每一次启动时生效)|节点1时默认为true,节点非1时为false|true|0.3.0|
|RNACOS_RAFT_JOIN_ADDR|是否当做节点加入对应的主节点,LeaderIp:GrpcPort；只在第一次启动时生效|空|127.0.0.1:9848|0.3.0|
|RUST_LOG|日志等级:debug,info,warn,error;所有http,grpc请求都会打info日志,如果不观注可以设置为error减少日志量|info|error|0.3.0|


启动配置方式可以参考： [运行参数说明](https://heqingpan.github.io/rnacos/deplay_env.html)

【集群部署】

集群部署参考： [集群部署](https://heqingpan.github.io/rnacos/cluster_deploy.html)


### 二、运行nacos 应用

服务启动后，即可运行原有的 nacos 应用。

### 配置中心http api例子

```
# 设置配置
curl -X POST 'http://127.0.0.1:8848/nacos/v1/cs/configs' -d 'dataId=t001&group=foo&content=contentTest'

# 查询
curl 'http://127.0.0.1:8848/nacos/v1/cs/configs?dataId=t001&group=foo'

```

### 注册中心http api例子

```
# 注册服务实例
curl -X POST 'http://127.0.0.1:8848/nacos/v1/ns/instance' -d 'port=8000&healthy=true&ip=192.168.1.11&weight=1.0&serviceName=nacos.test.001&groupName=foo&metadata={"app":"foo","id":"001"}'

curl -X POST 'http://127.0.0.1:8848/nacos/v1/ns/instance' -d 'port=8000&healthy=true&ip=192.168.1.12&weight=1.0&serviceName=nacos.test.001&groupName=foo&metadata={"app":"foo","id":"002"}'

 curl -X POST 'http://127.0.0.1:8848/nacos/v1/ns/instance' -d 'port=8000&healthy=true&ip=192.168.1.13&weight=1.0&serviceName=nacos.test.001&groupName=foo&metadata={"app":"foo","id":"003"}'

# 查询服务实例

curl "http://127.0.0.1:8848/nacos/v1/ns/instance/list?&namespaceId=public&serviceName=foo%40%40nacos.test.001&groupName=foo&clusters=&healthyOnly=true"

```


具体的用法参考 [nacos.io](https://nacos.io/zh-cn/docs/sdk.html) 的用户指南。

### nacos client sdk

#### java nacos sdk

**nacos-client**

```xml
<dependency>
    <groupId>com.alibaba.nacos</groupId>
    <artifactId>nacos-client</artifactId>
    <version>${nacos.version}</version>
</dependency>
```

|协议|验证过版本|推荐版本|
|--|--|--|
|grpc协议(2.x)|2.1.0|>2.1.x|
|http协议(1.x)|1.4.1|>1.4.x|


#### go nacos sdk

**nacos-sdk-go**

```
nacos-sdk-go/v2 v2.2.5
```

|协议|验证过版本|推荐版本|
|--|--|--|
|grpc协议(2.x)|2.2.5|>=2.2.5|

#### rust nacos sdk

**nacos-sdk-rust**

```
nacos-sdk = "0.3.3"
```

|协议|验证过版本|推荐版本|
|--|--|--|
|grpc协议|0.3.3|>=0.3.3|


**nacos_rust_client**

```
nacos_rust_client = "0.3.0"
```

|协议|验证过版本|推荐版本|
|--|--|--|
|同时支持http协议与grpc协议|0.3.0|>=0.3.0|
|http协议|0.2.2|>=0.2.2|


[其它语言](https://nacos.io/zh-cn/docs/other-language.html)

[open-api](https://nacos.io/zh-cn/docs/open-api.html)

### 三、控制台管理

从0.4.0版本开始，支持独立端口号的新控制台。新控制台有完备的用户管理、登陆校验、权限控制，支持对外网暴露。

启动服务后可以在浏览器通过 `http://127.0.0.1:10848/` 访问r-nacos新控制台。 
老控制台`http://127.0.0.1:8848/` 也仍然可用，不过后继会考虑废弃。老控制台不需要登陆、不支持用户管理。


主要包含用户管理、命名空间管理、配置管理、服务管理、服务实例管理。

1、用户登陆

在新控制台打开一个地址，如果检测到没有登陆，会自动跳转到登陆页面。
一个用户连续登陆失败5次，会被锁定1个小时。这个次数可以通过启动参数配置。

<img style="width: 400px;" width="400" src="https://github.com/heqingpan/rnacos/raw/master/doc/assets/imgs/20231223220425.png" />

2、用户管理

![](https://github.com/heqingpan/rnacos/raw/master/doc/assets/imgs/20231223222325.png)

系统会默认创建一个名为`admin`的用户，密码为`admin`。 

进去控制台后可按需管理用户。 

用户角色权限说明：

1. 管理员: 所有控制台权限
2. 开发者：除了用户管理的所有控制台权限
3. 访客：只能查询配置中心与注册中心的数据，没有编辑权限。


**注意：** 对外暴露的nacos控制台端口前，建议增加一个自定义管理员，把admin用户删除或禁用。


3、配置管理

配置列表管理

![](https://github.com/heqingpan/rnacos/raw/master/doc/assets/imgs/20230506155441.png)

新建、编辑配置

![](https://github.com/heqingpan/rnacos/raw/master/doc/assets/imgs/20230506155545.png)

4、服务列表管理

![](https://github.com/heqingpan/rnacos/raw/master/doc/assets/imgs/20230506155133.png)

5、服务实例管理

![](https://github.com/heqingpan/rnacos/raw/master/doc/assets/imgs/20230506155158.png)

6、命名空间管理

![](https://user-images.githubusercontent.com/1174480/268299574-4947b9f8-79e1-48e2-97fe-e9767e26ddc0.png)



## 功能说明

这里把 nacos 服务的功能分为三块
1、面向 SDK 的功能
2、面向控制台的功能
3、面向部署、集群的功能

每一块做一个对nacos服务的对比说明。

### 一、面向 SDK 的功能

访问认证：

1. 有提供获取认证token的接口
2. 实际请求暂不支持认证，都算认证通过。

配置中心：

1. 支持配置中心的基础功能、支持维护配置历史记录
2. 兼容配置中心的SDK协议
3. 暂不支持灰度发布、暂不支持tag隔离

注册中心：

1. 支持注册中心的基础功能
2. 兼容配置中心的SDK协议
3. 暂不支持1.x的 udp 实例变更实时通知，只支持 2.x 版本grpc实例变更实时通知 。最开始的版本也有支持过udp实例变更 通知，后面因支持 grpc 的两者不统一，就暂时去掉，后继可以考虑加回去。

### 二、面向开发、管理员的控制台的功能

控制台：
1. 支持使用独立的控制台端口提供对外网服务。

用户管理：

1. 支持管理用户列表
2. 支持用户角色权限管理
3. 支持用户密码重置

命名空间：

1. 支持管理命名空间列表
2. 支持切换命名空间查询配置、服务数据。

配置中心：

1. 支持配置中心信息管理
1. 支持配置导入、导出,其文件格式与nacos兼容
2. 支持配置历史记录查看与恢复
3. 暂不支持tag的高级查询
4. 暂不支持查询配置监听记录

服务中心：

1. 支持注册中心的服务、服务实例管理
2. 暂不支持查询监听记录

### 三、面向部署、集群的功能

1. 支持单机部署
2. 支持集群部署。集群部署配置中心数据使用raft+节点本地存储组成的分布式存储，不需要依赖mysql。具体参考 [集群部署说明](https://heqingpan.github.io/rnacos/cluster_deploy.html)


## 性能


|模块|场景|单节点qps|集群qps|总结|
|--|--|--|--|--|
|配置中心|配置写入,单机模式|1.5万|-||
|配置中心|配置写入,集群模式|1.8千|1.5千|接入raft后没有充分优化,待优化,理论上可接近单机模式|
|配置中心|配置查询|8万|n*8万|集群的查询总qps是节点的倍数|
|注册中心|服务实例注册,http协议|1.2万|1.0万|注册中心单机模式与集群模式写入的性能一致|
|注册中心|服务实例注册,grpc协议|1.2万|1.2万|grpc协议压测工具没有支持，目前没有实际压测，理论不会比http协议低|
|注册中心|服务实例心跳,http协议|1.2万|1.0万|心跳是按实例计算和服务实例注册一致共享qps|
|注册中心|服务实例心跳,grpc协议|8万以上|n*8万|心跳是按请求链接计算,且不过注册中心处理线程,每个节点只需管理当前节点的心跳，集群总心跳qps是节点的倍数|
|注册中心|查询服务实例|3万|n*3万|集群的查询总qps是节点的倍数|

**注：** 具体结果和压测环境有关

详细信息可以参考
[性能与容量说明](https://heqingpan.github.io/rnacos/performance.html)


## r-nacos架构图

单实例：

![](https://github.com/heqingpan/rnacos/raw/master/doc/assets/imgs/rnacos_L2_0.1.4.svg)

前端应用因依赖nodejs,所以单独放到另一个项目 [r-nacos-console-web](https://github.com/heqingpan/rnacos-console-web) ,再通过cargo 把打包好的前端资源引入到本项目,避免开发rust时还要依赖nodejs。


r-nacos架构设计参考： [架构](https://heqingpan.github.io/rnacos/architecture.html)


## 使用登记

使用r-nacos的同学，欢迎在[登记地址](https://github.com/heqingpan/rnacos/issues/32) 登记，登记只是为了方便产品推广。

## 联系方式

> R-NACOS微信沟通群：先加微信(添加好友请备注'r-nacos')，再拉进群。

<img style="width: 200px;" width="200" src="https://github.com/heqingpan/rnacos/raw/master/doc/assets/imgs/wechat.jpg" alt="qingpan2014" />

