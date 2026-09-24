# Proposal: 生成配置强制执行严格 DNS 与出口隐私

状态：`accepted`

## 现状与问题

提案建立时产品只生成 `ProfileDraft`，尚未接通 Node/Rule IR、DNS/TUN、YAML serializer、Mihomo 验证或文件导出。当前已在本 change 中接通严格 Node/Rule/Provider IR、确定性 YAML 与静态回读；绑定 sidecar、文件导出和运行态证据仍未接通。

如果未来直接把当前草案扩展为 YAML，可能出现以下用户可观察风险：

- 普通 UDP/TCP 53 查询绕过 Mihomo；
- DoH 已加密但通过 `DIRECT` 建连，解析器仍看到用户原始出口 IP；
- 国内直连、IPv6 或未进入 TUN 的应用流量暴露原始 IP；
- 代理节点使用域名时，为建立首个代理连接而直连解析，形成 bootstrap 泄漏；
- 规则或节点 provider 在启动、刷新时绕过代理下载；
- 仅凭静态 YAML 就错误显示“DNS/原始 IP 已保护”。

## 目标

1. 把严格隐私作为默认且不可被简单模式绕开的编译意图；
2. 最终 YAML 强制包含 `fake-ip`、DNS 劫持、`respect-rules`、代理出口加密 DNS、TUN `strict-route`、IPv6 默认关闭和零 `DIRECT` 用户流量；
3. 对节点域名 bootstrap、remote provider 更新和其他必须直连的辅助流量采用 fail-closed：没有受保护路径就阻断编译；
4. UI 在草案阶段展示这些“编译硬约束”，但不把它表述成已经生成或已经防泄漏；
5. 将静态生成、Mihomo 加载、DNS 路径、规则命中和公网出口分别验证。

## 非目标

- 只有 serializer 与静态回读成功才声明“YAML 已生成”；当前不声明“已导出”或运行态隐私已验证；
- 不直接修改 Clash Verge 内部配置或数据库；
- 不把浏览器 WebRTC、应用绕过系统代理或客户端 GUI 覆盖问题伪装成单个 YAML 文件可以独立证明的结果；
- 不引入真实订阅、节点、DNS token 或设备信息作为 fixture。

## 影响能力与实现

- `profile-compilation`：草案增加严格隐私编译契约，strict compiler 已消费该契约；
- `security-privacy`：增加 DNS、bootstrap、provider 和零直连的安全边界；
- `validation-export`：增加静态隐私校验和 DNS/公网出口证据分层；
- `desktop-experience`：预览明确显示“严格隐私约束”，静态编译成功后单独显示“YAML 已生成”；
- 当前实现文件：profile/compile domain、application/IPC、前端 contract/workspace/preview 与测试；
- 后续实现文件：绑定 Mihomo validation、export adapters 和运行态 E2E。

## 安全、兼容与回滚

- 严格模式会拒绝带 `DIRECT` 目标、无保护域名 bootstrap 或直连 remote provider 的配置；这是 fail-closed，不做静默降级；
- 默认关闭 IPv6 会牺牲 IPv6 访问；只有同时证明 IPv6 TUN、DNS 和节点出口闭环后才能启用；
- 强制 TUN `strict-route` 可能影响局域网访问和虚拟机网络，必须在导出前提示；
- 当前响应 DTO 新增字段；浏览器 adapter 与 Rust IPC 必须同步；
- 当前目录没有 Git 元数据，本次修改前副本保存在 `/tmp/mihomo-studio-before-strict-dns-privacy-20260814T2230`。

## 验收场景

### 场景：草案表达隐私硬约束

- **WHEN** 用户生成简单或完全模式草案
- **THEN** 草案包含严格隐私策略摘要
- **AND** 明确要求代理 DNS、DNS 劫持、TUN strict route、禁用 IPv6 和禁止 DIRECT
- **AND** UI 只显示“编译约束”，不显示“已防泄漏”。

### 场景：编译器发现直连路径

- **GIVEN** 规则 target 为 `DIRECT`，或 remote provider 使用直连更新
- **WHEN** 进入严格隐私编译
- **THEN** 编译失败并返回稳定、无秘密的错误类别
- **AND** 不生成或覆盖 YAML。

### 场景：节点域名需要 bootstrap

- **GIVEN** 代理节点 server 是域名
- **AND** 没有外层代理、已验证的本地映射或其他受保护 bootstrap
- **WHEN** 进入严格隐私编译
- **THEN** 编译失败
- **AND** 不用系统 DNS 或直连 DoH 静默降级。

### 场景：静态配置通过

- **WHEN** YAML serializer、回读和绑定 Mihomo 加载均成功
- **THEN** 状态最多为“Mihomo 已验证”
- **AND** 仍分别等待 Clash Verge 加载、DNS 路径、规则命中和公网出口证据。

## 证据要求

| 层级 | 证据 |
|---|---|
| 草案 | DTO、Rust/TS 单元测试和 UI 文案 |
| 已生成 | YAML 回读与隐私不变量静态检查 |
| Mihomo 已验证 | 固定版本内核实际加载 |
| 已导出 | 私有权限和原子替换 |
| Clash Verge 已加载 | 当前运行配置读取 |
| DNS 路径 | Controller/连接或抓包证明 DNS 连接使用指定代理，且无意外 53/DoH 直连 |
| 规则命中 | 目标流量的规则与连接归因 |
| 最终出口 | IPv4/IPv6/WebRTC 相关目标流量不出现本地原始出口 |

## 当前实施证据

- strict Node IR 只接受 IPv4 literal Mihomo YAML，域名/IPv6/旁路字段 fail-closed；
- catalog `DIRECT` 在进入 Rule IR 前显式改写为 `节点选择`，remote provider 固定 `proxy: 节点选择`；
- YAML golden/round-trip 覆盖 fake-ip、代理 DoH、DNS hijack、TUN strict route、IPv6 关闭和唯一最终 MATCH；
- `npm run test:mihomo-fixture` 使用官方 Mihomo `v1.19.29` darwin arm64 压缩包 SHA-256 `4dc25df9e899f14161911302a8ee5fc9e202ed9c976fc405bf82c50ff27466ca`，已对脱敏 golden YAML 执行 `-t` 成功；该可复现 fixture gate 不替代应用内绑定 sidecar 或用户配置验证。
