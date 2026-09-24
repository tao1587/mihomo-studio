# Delta: 桌面体验

## ADDED Requirements

### Requirement: 隐私约束预览

草案预览 SHALL 显示严格隐私编译约束摘要，包括代理 DNS、DNS 劫持、TUN strict route、IPv6 状态、DIRECT 状态、bootstrap 与 provider 更新边界。

#### Scenario: 当前尚未生成 YAML

- **WHEN** UI 展示隐私摘要
- **THEN** 标题或说明使用“编译硬约束”
- **AND** 不显示“DNS 已防泄漏”“原始 IP 已隐藏”或其他运行态成功结论。
