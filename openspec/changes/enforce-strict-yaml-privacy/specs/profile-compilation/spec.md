# Delta: 配置草案与编译

## ADDED Requirements

### Requirement: 严格隐私编译意图

每个 `ProfileDraft` SHALL 包含不可缺省的严格隐私编译意图，至少声明：

- DNS 使用 `fake-ip`；
- DNS 查询只允许通过代理出口；
- 必须启用 DNS hijack 与 TUN `strict-route`；
- 未完成 IPv6 代理闭环验证前禁用 IPv6；
- 用户流量不得使用 `DIRECT`；
- 节点域名必须具备受保护 bootstrap；
- remote rule/proxy provider 只允许通过代理更新。

该字段只描述后续编译硬约束，不表示 YAML 已生成或隐私已验证。

#### Scenario: 构建简单草案

- **WHEN** 构建简单模式草案
- **THEN** 返回严格隐私编译意图
- **AND** 国内流量层次不得承诺最终 YAML 使用 `DIRECT`。

### Requirement: 严格隐私 fail-closed 编译

最终编译器 SHALL 在发现 `DIRECT` 用户规则、未保护域名 bootstrap、直连 provider 更新或可能旁路的 IPv6 路径时失败，不得静默生成降级 YAML。
