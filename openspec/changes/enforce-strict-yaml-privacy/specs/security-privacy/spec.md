# Delta: 安全与隐私

## ADDED Requirements

### Requirement: DNS 与原始出口隐私

严格隐私配置 SHALL：

- 让应用 DNS 进入 Mihomo 内部 DNS；
- 让上游加密 DNS 连接通过指定代理策略；
- 拒绝把系统 DNS、普通 53 或直连 DoH 作为 fallback；
- 拒绝无受保护路径的节点域名 bootstrap；
- 拒绝 remote provider 的直连下载或刷新；
- 使用 TUN strict route 并在闭环验证前禁用 IPv6；
- 禁止用户目标流量落到 `DIRECT`。

#### Scenario: 加密但直连的 DoH

- **GIVEN** DoH URL 使用 HTTPS
- **AND** 连接策略为 `DIRECT`
- **WHEN** 严格隐私检查运行
- **THEN** 检查失败
- **AND** 不把“已加密”解释为“未暴露原始出口”。

### Requirement: 隐私事实状态

静态 YAML 只能证明隐私字段与引用满足编译契约。只有外部客户端加载、DNS 路径、规则命中和公网出口均有运行态证据时，UI 才可显示对应隐私验证状态。
