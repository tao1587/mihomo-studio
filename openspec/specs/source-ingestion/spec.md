# Capability: 节点来源接入

| 字段 | 值 |
|---|---|
| ID | `source-ingestion` |
| 状态 | `partial` |
| IPC | `inspect_subscription`, `inspect_node_text`, `convert_node_text` |

## Purpose

安全获取和识别订阅或直接节点内容，返回不含秘密的结构摘要，并为后续 Node IR 提供稳定输入边界。

## Current State

已实现：

- `system` / `direct` 两种抓取路径；
- 4 个 User-Agent 顺序探测；
- URI、Base64 URI、Mihomo `proxies` YAML 与 `proxy-providers` YAML 识别；
- URI/YAML 节点内容指纹和单来源精确去重；
- HTTP(S)、重定向、超时和 8 MiB 上限；
- `SourceInspectionSummary` 安全摘要。
- strict compiler 会在用户主动生成时重新获取订阅 material，或使用当前 React 会话内的直接节点 material；原文不进入摘要、日志或持久化；
- 本地可枚举 Mihomo `proxies` YAML、标准 VLESS URI 和 Base64 VLESS URI 列表可进入严格 Node IR；
- 来源摘要独立报告 `formatReadyForCompilation`，不再把“已识别出节点”当成“格式已接通编译”。
- VLESS URI/Base64 VLESS 在加入工作区前复用权威转换器执行参数级预检；预检失败时保留安全统计，但不再标记为可编译。
- 独立“节点转换”功能使用协议感知的 `convert_node_text` IPC，生成仅含 `proxies:` 的 Clash/Mihomo 节点 YAML 片段；支持单条、多行与 Base64 VLESS 文本，以及一份单 Peer WireGuard 客户端文本。VLESS 精确去重并处理重名，WireGuard 严格映射标准字段。
- 单个 IP 服务器节点的转换结果还可包含去敏内置模版组成的完整 YAML 和 IP 文件名；多节点或域名服务器仅返回片段，不做 DNS 查询。

尚未实现：VMess/Trojan/SS 等其他 URI 协议转换、provider-only resolver、跨来源完整 provenance、持久化来源、last-known-good、凭据库和文件导入。

## Requirements

### Requirement: 输入通道

系统 SHALL 接受 HTTPS 订阅 URL 和本地直接节点文本。HTTP 只允许 loopback 地址；远程 URL authority 不得携带用户名或密码。

#### Scenario: 远程 HTTP 订阅

- **WHEN** 用户提交非 loopback 的 HTTP URL
- **THEN** 请求在发出前被拒绝
- **AND** 错误不回显完整 URL

### Requirement: 受限抓取

订阅请求 SHALL 设置总超时、响应上限和重定向上限；SHALL 阻止 HTTPS 降级、危险 scheme、authority 凭据和远程到 loopback 的重定向。

当前约束：20 秒、8 MiB、最多 5 次重定向。

#### Scenario: 超大响应

- **WHEN** Content-Length 或流式累计字节超过 8 MiB
- **THEN** 抓取终止并返回稳定错误类别
- **AND** 响应内容不进入错误详情

### Requirement: 内容驱动识别

格式识别 SHALL 依据响应内容，而不是文件扩展名或 Content-Type。探测顺序 SHALL 支持 Mihomo YAML、provider YAML、URI、多行 URI 和 Base64 包装内容；HTML 错误页不得计为节点来源。

#### Scenario: User-Agent 差异

- **GIVEN** 前几个 User-Agent 返回不可识别内容
- **WHEN** 后续 User-Agent 返回有效订阅
- **THEN** 返回成功摘要和回退警告
- **AND** 只记录安全的 User-Agent 标签，不记录订阅内容

### Requirement: 精确节点身份

节点显示名称 SHALL 不参与精确内容指纹。URI 查询参数 SHALL 规范化排序；YAML proxy 的顶层 `name` SHALL 在指纹中省略。

相同 endpoint 但凭据、传输、TLS 或其他连接参数不同的节点不得合并。

#### Scenario: 节点改名

- **GIVEN** 两个节点只在显示名上不同
- **WHEN** 解析单个来源
- **THEN** 它们产生同一内容指纹并计为精确重复

### Requirement: Resolver 边界

本地 parser 不能枚举但 Mihomo 可能支持的内容 SHALL 返回 `requiresMihomoResolver=true`，不得伪造节点数量或编译就绪。

#### Scenario: provider YAML

- **WHEN** YAML 只含 `proxy-providers` 而没有本地可枚举 `proxies`
- **THEN** 摘要标记需要 resolver
- **AND** `resolvedNodeCount` 不因该来源被增加

### Requirement: 格式编译就绪状态

来源摘要 SHALL 使用 `formatReadyForCompilation` 区分“内容已识别”和“当前编译器已安全映射”。Mihomo `proxies` YAML 可以按当前格式边界标记就绪；纯 VLESS URI 与 Base64 VLESS URI 只有在权威 VLESS 转换器对全部节点完成参数级预检后才可标记就绪。混合协议、未映射协议、参数预检失败、provider-only 或未知内容不得标记就绪。

#### Scenario: 已识别但协议未映射

- **WHEN** URI 列表含当前未映射协议
- **THEN** 节点和协议统计仍可显示
- **AND** `formatReadyForCompilation=false`
- **AND** 草案不得显示“可进入编译”。

#### Scenario: VLESS 已识别但参数预检失败

- **GIVEN** URI 列表中的多个 VLESS 节点包含未知参数、冲突别名或不兼容 transport/security 选项
- **WHEN** 检查订阅或直接节点文本
- **THEN** 摘要保留脱敏节点数量和协议统计
- **AND** `formatReadyForCompilation=false`
- **AND** 警告按来源内节点序号汇总稳定问题类别，不回显 URI、节点名、参数名或参数值。

### Requirement: VLESS 分享链接转换

VLESS URI SHALL 按分享链接字段转换为 Mihomo VLESS Node IR。重复查询参数、未知安全参数或无法等价映射的传输选项 SHALL fail-closed；错误不得回显完整 URI 或参数值。

同一 URI 来源包含多个失败节点时，转换器 SHALL 继续预检其余节点并一次返回总问题数。错误详情最多列出前 8 个节点位置和稳定类别，剩余项目只报告数量；不得把用户控制的参数名或值写入错误。

`security=reality` 链接中的非空 `spx` 与 `pqv` 是显式的 Mihomo 目标兼容特例：当前目标内核没有等价字段，编译器 SHALL 消费但不写入 YAML，并在无秘密报告中按受影响节点数量提示 REALITY 能力降级。该特例不得用于非 REALITY 节点，也不得扩展为忽略其他未知参数。

订阅导出的等值 `sni`/`servername` SHALL 归一化为单个 Mihomo `servername`；值冲突时 SHALL 失败。`type=tcp`/`raw` 上误附带的已知 gRPC mode `gun`/`multi`/`guna` SHALL 被消费但不写入 YAML，并计入目标兼容归一化警告；未知 mode 继续 fail-closed。

### Requirement: 独立节点片段转换

系统 SHALL 提供协议感知、独立于完整配置编译的节点片段转换：输入支持单条、多行或 Base64 VLESS URI 文本，以及一份包含一个 `[Interface]` 和一个 `[Peer]` 的标准 WireGuard 客户端文本；输出为可合并进 Clash/Mihomo 配置的 `proxies:` YAML。VLESS 操作 SHALL 复用严格编译器的权威字段映射，不得复制第二套参数白名单。

桌面入口 SHALL 位于独立 `node-converter` 功能切片，不得嵌入来源编辑器或复用配置生成的直接节点状态。

输出 SHALL 为重名节点分配稳定 `[n]` 后缀，并按不含显示名称的精确连接指纹去重。结果 SHALL 报告输出节点数、重复数、目标兼容归一化数和无秘密警告。任何节点转换失败时 SHALL 不返回部分 YAML。

单节点服务器为 IP literal 时，转换结果 SHALL 额外包含内置完整模版 YAML。模版只替换唯一节点及引用它的策略组成员，生成节点名 SHALL 为服务器 IP；原模版节点名称、域名和凭据不得进入内置资源。域名节点即使显示名像 IP，也不得据此推断服务器 IP。

WireGuard SHALL 映射 `PrivateKey`、IPv4/可选 IPv6 `Address`、可选 `DNS`、`PublicKey`、`Endpoint`、`AllowedIPs` 与可选 `PersistentKeepalive`；`Address` 的 CIDR 前缀 SHALL 原样保留其规范化语义，DNS 存在时 SHALL 同时启用节点级 `remote-dns-resolve`。没有名称的配置 SHALL 使用稳定非秘密名称 `WireGuard 1`。缺少必要字段、字段重复、值格式无效、包含多个 Peer 或包含未映射字段时 SHALL fail-closed，不返回部分 YAML，错误不得回显字段值、Endpoint、地址或密钥。

独立转换支持 WireGuard 不代表 WireGuard INI 已接入来源检查或完整配置编译，不得自动标记 `formatReadyForCompilation=true`。

#### Scenario: 单条 REALITY VLESS 转换

- **GIVEN** 一条包含 REALITY public key、short id、SNI、client fingerprint 和 TCP transport 的 VLESS URI
- **WHEN** 调用 `convert_node_text`
- **THEN** 输出一个位于 `proxies:` 下的 Mihomo VLESS 节点
- **AND** 使用 `servername`、`client-fingerprint`、`reality-opts` 与 `network` 目标字段。

#### Scenario: 多条 VLESS 的去重与重名

- **GIVEN** 输入包含完全重复节点以及两个同名但连接参数不同的节点
- **WHEN** 转换节点片段
- **THEN** 完全重复节点只输出一次并计入重复数
- **AND** 同名不同节点分别保留基础名和 `[2]` 后缀。

#### Scenario: 单 Peer WireGuard 客户端配置

- **GIVEN** 一份包含私钥、IPv4 Address、DNS、单个 Peer 公钥、Endpoint、AllowedIPs 和 keepalive 的 WireGuard 客户端文本
- **WHEN** 调用 `convert_node_text`
- **THEN** 输出一个使用 `peers` 完整写法的 Mihomo WireGuard 节点
- **AND** 完整密钥只保留在敏感 YAML，不进入无秘密状态摘要。

#### Scenario: 多 Peer WireGuard 拒绝

- **GIVEN** WireGuard 输入包含多个 `[Peer]`
- **WHEN** 调用转换
- **THEN** 转换以稳定类别全量失败
- **AND** 不返回部分 YAML 或回显任一 Peer 内容。

### Requirement: 编译 material 生命周期

节点/订阅原文 SHALL 只保留在当前 React 状态、当前 IPC 参数、订阅响应内存和 Node IR 编译调用中。DTO 不得实现或使用可能意外打印 secret value 的调试输出；编译错误不得包含 URL、节点名称、server 或凭据。

独立转换输出包含节点凭据，SHALL 只保留在当前 React 状态、当前 IPC 结果和用户主动复制的剪贴板中；不得自动写文件、日志、缓存或持久化。独立转换页 MAY 在用户主动转换成功后直接显示完整敏感 YAML，但不得把它放入 live-region。

### Requirement: 安全摘要

返回 UI 的摘要 SHALL 只包含脱敏 host 标签、格式、节点/重复数量、协议分布、安全 User-Agent 标签、警告、resolver 状态和格式编译就绪状态。不得包含 path、query、fragment、节点显示名、UUID、密码或原始内容。

## Contracts

`SubscriptionInspectionRequest`：

- `url: String`：只在当前调用和请求内存使用；
- `fetchRoute: "system" | "direct"`。

`SourceInspectionSummary`：

- `safeLabel`；
- `sourceFormat`；
- `nodeCount` / `duplicateCount`；
- `protocols[]`；
- `userAgent`；
- `warnings[]`；
- `requiresMihomoResolver`。

`ConvertNodeTextRequest`：

- `content: String`：只在当前调用内存使用，不实现 Debug 输出。

`ConvertNodeTextResult`：

- `yaml`：敏感 `proxies:` YAML 片段；
- `templateYaml`：仅单个 IP 服务器节点可用的敏感内置模版完整 YAML，否则为 `null`；
- `templateFileName`：仅模版可用时返回以服务器 IP 命名的建议 YAML 文件名，否则为 `null`；
- `nodeCount` / `duplicateNodeCount`；
- `compatibilityNormalizationCount`；
- `warnings[]`：只含数量与稳定兼容提示。

## Implementation Map

| 路径 | 责任 |
|---|---|
| `src-tauri/src/domain/source/model.rs` | 请求/摘要模型、URL 值对象、抓取路径和 typed error |
| `src-tauri/src/domain/source/parser.rs` | 内容识别、指纹、单来源去重 |
| `src-tauri/src/application/source.rs` | UA 探测与来源检查用例 |
| `src-tauri/src/application/ports.rs` | Subscription gateway port |
| `src-tauri/src/infrastructure/source/reqwest_gateway.rs` | 受限 HTTP adapter |
| `src-tauri/src/interface/ipc/source.rs` | Tauri source commands |
| `src-tauri/src/domain/compile/model.rs` | 协议感知的独立转换请求/敏感结果契约与 typed error |
| `src-tauri/src/domain/compile/node.rs` | YAML/VLESS URI/Base64 VLESS Node IR、WireGuard INI 转换、分享字段映射与 fail-closed bootstrap 边界 |
| `src-tauri/src/domain/compile/template.rs` | 单个 IP 节点与去敏内置模版组合 |
| `src-tauri/src/domain/compile/templates/clash-single-node.yml` | 不含原节点凭据和名称的内置模版骨架 |
| `src-tauri/src/domain/compile/serializer.rs` | `proxies:` 节点片段与完整配置 YAML 序列化 |
| `src-tauri/src/application/compile.rs` | 当前编译调用的订阅 material 获取 |
| `src/features/source-inspection/` | 来源输入和摘要展示 |
| `src/features/node-converter/` | 独立转换输入、IPC 调用和敏感结果展示 |
| `src/shared/api/studio.ts` | 前端 IPC adapter |
| `src/shared/contracts.ts` | TypeScript 请求/响应类型 |

## Verification

```bash
cd src-tauri && cargo test source::
npm test
npm run test:mihomo-fixture
```

集成验证需要使用仅含占位值的 loopback fixture；报告不得保存真实订阅 URL 或节点凭据。
