# Capability: 配置草案与编译

| 字段 | 值 |
|---|---|
| ID | `profile-compilation` |
| 状态 | `partial` |
| IPC | `build_profile_draft`, `compile_strict_profile` |

## Purpose

把模式、已解析节点数量和规则来源选择转换为可解释的策略结构，并逐步演进为确定性的 Mihomo YAML 编译器。

## Current State

当前实现包含两个相互独立的阶段：

- `ProfileDraft`：策略组名称、规则层次、唯一最终规则、严格隐私编译意图、警告和 `readyForCompilation`；
- `compile_strict_profile`：把本地可枚举的 Mihomo `proxies` YAML、VLESS URI/Base64 VLESS URI、catalog rule set 和用户意图编译为确定性严格隐私 YAML，并在返回前完成 IR 与 YAML 回读检查。

Node IR 当前接受 IPv4 literal server，或带用户确认 IPv4 `bootstrapMappings` 的域名 server Mihomo YAML/VLESS 分享链接节点。无映射域名、IPv6 server、`interface-name`、`routing-mark`、`dialer-proxy`、未映射协议/参数和 provider-only 来源均 fail-closed；VLESS URI 来源会一次预检全部节点并返回有上限的脱敏问题汇总；REALITY `spx`/`pqv` 按 Mihomo 目标边界显式省略并报告能力降级，其他协议/provider resolver 仍是后续能力。

浏览器预览由 `src/features/profile-builder/model/catalog.ts` 提供近似实现；Tauri 运行时以 Rust domain service 结果为准。

## Requirements

### Requirement: 请求校验

`BuildDraftRequest` SHALL：

- 只接受 `simple` 或 `full`；
- 只接受 catalog 已知且可选择的 source ID；
- 拒绝同一 exclusive group 的多选；
- GitHub mirror 非空时必须是 HTTPS URL；
- 将节点来源总数与已解析节点数作为不同输入。

#### Scenario: 未解析来源

- **GIVEN** `inputSourceCount > 0`
- **AND** `resolvedNodeCount = 0`
- **WHEN** 构建草案
- **THEN** 草案有效但 `readyForCompilation=false`
- **AND** 警告说明需要 Mihomo resolver

### Requirement: 策略组草案

简单模式 SHALL 至少形成 `节点选择`、`自动选择`、`故障转移`、`AI / Claude`、`流媒体`、`广告拦截` 和最终 `其他兜底`。完全模式可以增加服务组，但 `其他兜底` 必须是最后一个组。

#### Scenario: 简单模式草案

- **WHEN** 使用有效来源构建简单模式草案
- **THEN** `其他兜底` 是最后一个策略组
- **AND** 最终规则为 `MATCH,其他兜底`

### Requirement: 最终规则

草案和最终编译结果 SHALL 只有一个 `MATCH`，且它必须是最后一条规则。其他规则不得在 `MATCH` 后产生。

### Requirement: 严格隐私编译意图

简单和完全模式 SHALL 共享同一个不可缺省的严格隐私契约：

- DNS 模式为 `fake-ip`，DNS 上游只允许代理出口；
- 最终 YAML 必须启用 DNS hijack 与 TUN `strict-route`；
- 未完成 IPv6 代理闭环验证前关闭 IPv6；
- 用户流量不得使用 `DIRECT`，国内规则也必须使用代理出口；
- 节点域名必须具备受保护 bootstrap；
- remote rule/proxy provider 只允许通过代理更新。

该字段只描述后续编译硬约束，不表示 YAML 已生成、客户端已加载或防泄漏已验证。

#### Scenario: 简单模式隐私草案

- **WHEN** 构建简单模式草案
- **THEN** `privacy.level=strict`
- **AND** DNS、TUN、IPv6、DIRECT、bootstrap 和 provider 更新约束均为机器可读字段
- **AND** 规则层次不得包含“国内直连”。

#### Scenario: 编译发现非受保护出口

- **GIVEN** Rule/Provider/Node IR 包含 `DIRECT` 用户流量、直连 provider 更新或无保护域名 bootstrap
- **WHEN** 进入严格隐私编译
- **THEN** 编译失败
- **AND** 不静默生成降级 YAML。

### Requirement: 编译就绪语义

只有至少一个输入来源、至少一个已解析节点和至少一个规则 source 时，当前草案才可标记 `readyForCompilation=true`。该字段只表示输入结构满足下一阶段条件，不表示 YAML 已生成。

### Requirement: 冲突优先于去重

未来 Rule IR 遇到相同 matcher/value 但不同 target 时 SHALL 生成冲突；不得作为重复项删除。自动语义去重只允许在 target 和 flags 相同、覆盖关系可证明时进行，并保留 provenance。

#### Scenario: 规则目标不同

- **GIVEN** 两条规则匹配同一域名
- **AND** target 分别为 `DIRECT` 和 `AI / Claude`
- **WHEN** 编译 Rule IR
- **THEN** 两条规则进入冲突报告
- **AND** 任何删除都需要显式优先级和可追踪决策

### Requirement: 确定性输出

同一版本的 catalog、来源内容 hash、用户意图和编译器版本 SHALL 产生字节稳定或语义稳定的 YAML 与报告。输出必须先完成引用、组图、规则顺序和唯一 `MATCH` 检查。

### Requirement: 严格 Node IR

编译器 SHALL 从 Mihomo `proxies` YAML 或标准 VLESS URI/Base64 URI 构建 Node IR，按忽略显示名的连接字段指纹跨来源精确去重，并保证最终节点名唯一。当前严格模式接受 IPv4 literal server，或带一对一、用户确认 IPv4 bootstrap 映射的域名 server；无映射域名、IPv6 server、重复或未映射分享参数以及可能强制旁路的节点字段 SHALL 返回不含节点名称、地址或凭据的稳定错误。VLESS URI 解析不得在首个节点失败时停止；同一来源的问题总数 SHALL 一次报告，详情最多包含前 8 个节点位置与稳定类别，且不包含用户控制的参数名或值。REALITY 非空 `spx`/`pqv` 仅作为目标不支持扩展被消费，不写入 YAML；等值 `sni`/`servername` 被合并，TCP/raw 上的 `gun`/`multi`/`guna` mode 被省略。发生有损目标归一化时 SHALL 在报告中明确受影响节点数；冲突别名和其他未知字段继续失败。

#### Scenario: 多个 VLESS 节点参数失败

- **GIVEN** 同一 URI 来源有多个节点分别包含未知分享参数和不兼容 transport 选项
- **WHEN** 构建严格 Node IR
- **THEN** 编译一次返回问题总数及有上限的节点位置/类别
- **AND** 不需要用户逐次生成才能发现后续节点问题
- **AND** 错误不回显 URI、节点名称、server、UUID、参数名或参数值。

#### Scenario: VLESS Reality URI

- **GIVEN** VLESS URI 使用 IPv4 server、Reality、合法 UUID/public key 和受支持传输字段
- **WHEN** 进入严格 Node IR
- **THEN** 分享字段映射为 Mihomo `uuid`、TLS/Reality、transport options 与唯一节点名
- **AND** secret-bearing 字段只进入 Node IR 和最终 YAML。

#### Scenario: 节点 server 为域名且缺少映射

- **WHEN** 节点 `server` 不是 IP literal
- **THEN** 编译返回未保护 bootstrap 错误
- **AND** 错误只包含来源/节点序号，不回显域名。

#### Scenario: 节点 server 使用已验证 IPv4 映射

- **GIVEN** 节点 `server` 是域名
- **AND** `bootstrapMappings` 为该来源/节点提供 IPv4 literal
- **WHEN** 进入严格 Node IR
- **THEN** 节点保留域名连接身份
- **AND** serializer 把映射写入顶层 `hosts`
- **AND** 报告和错误不包含域名或映射值。

### Requirement: Rule 与 Provider IR

编译器 SHALL 从 catalog 选择当前模式和已选 source 对应的 rule set，保持确定性顺序和唯一 `MATCH`。catalog 中原始 `DIRECT` 意图在进入严格 Rule IR 前 SHALL 显式重写为 `节点选择`；remote provider SHALL 使用 HTTPS、受管相对路径、大小限制和 `proxy: 节点选择`。

内建 `GEOIP,CN` 规则在缺少本地固定 geodata 时 SHALL 从严格编译结果中省略，避免 Mihomo 启动时直连下载 geodata。

### Requirement: 严格 YAML 输出

生成 YAML SHALL 包含：

- 仅本机监听的 mixed port 与 DNS listener；
- 域名节点存在时，包含一对一的用户确认 IPv4 `hosts` 映射；
- `fake-ip`、`respect-rules=true`、禁止 system hosts、代理绑定的 IP-literal DoH；
- TUN `auto-route`、UDP/TCP 53 hijack 与 `strict-route`；
- 顶层和 DNS IPv6 关闭；
- 节点、无环策略组、代理下载 provider 和唯一最终规则。

输出成功状态为“已生成”，不得推断 Mihomo、Clash Verge、DNS 路径或公网出口已经验证。

## Contracts

`BuildDraftRequest` 当前字段：

- `mode`；
- `selectedRuleSourceIds[]`；
- `inputSourceCount`；
- `resolvedNodeCount`；
- `githubMirror`。

`ProfileDraft` 当前字段：

- 模式与来源/节点数量；
- `groups[]`、`ruleOrder[]`、`finalRule`；
- `privacy`：严格 DNS、TUN、IPv6、DIRECT、bootstrap 和 provider 更新约束；
- `warnings[]`；
- `readyForCompilation`。

`CompileProfileRequest` 当前字段：

- `mode`、`selectedRuleSourceIds[]`、`githubMirror`；
- `sources[]`：`subscription` 或 `nodes`、仅当前 IPC 使用的 secret value，以及 `system`/`direct` 抓取路径。
- `bootstrapMappings[]`：从 1 开始的来源/节点序号与用户确认 IPv4；仅当前 IPC 和 secret-bearing YAML 使用。

`CompileProfileResult` 当前字段：

- `yaml`：secret-bearing Mihomo YAML；
- `report`：生成状态、数量、无秘密 SHA-256、严格隐私标记、目标兼容降级和证据边界警告。

字段扩展必须同步 Rust domain/application contract、`src/shared/contracts.ts`、前端调用和 Spec index。

## Implementation Map

| 路径 | 责任 |
|---|---|
| `src-tauri/src/domain/profile/model.rs` | 请求、草案和 typed error |
| `src-tauri/src/domain/profile/service.rs` | 权威校验与纯草案领域服务 |
| `src-tauri/src/application/profile.rs` | Catalog port 与草案用例编排 |
| `src-tauri/src/interface/ipc/profile.rs` | Tauri profile command |
| `src-tauri/src/domain/compile/` | Node/Rule/Provider/Group IR、serializer、回读和严格隐私检查 |
| `src-tauri/src/application/compile.rs` | 订阅 material 解析与编译用例编排 |
| `src-tauri/src/interface/ipc/compile.rs` | Tauri strict compile command |
| `src/features/profile-builder/model/catalog.ts` | 浏览器模式草案 adapter |
| `src/features/profile-builder/model/catalog.test.ts` | 前端草案不变量 |
| `src/shared/contracts.ts` | 前端请求与草案 DTO |
| `src/app/useStudioWorkspace.ts` | 草案输入收集和用例调用 |

## Verification

```bash
npm test
cd src-tauri && cargo test profile::
cd src-tauri && cargo test compile::
```

当前已有 parser round-trip、严格隐私不变量、Mihomo YAML 与 VLESS URI 脱敏 golden file 测试。绑定 sidecar、跨来源完整 provenance 和其他协议/provider resolver 仍需追加。
