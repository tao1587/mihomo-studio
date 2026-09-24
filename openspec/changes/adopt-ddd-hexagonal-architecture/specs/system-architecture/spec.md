# Delta: system-architecture

## Modified Requirement: 分层与依赖方向

Rust 后端 SHALL 使用以下依赖规则：

```text
interface -> application -> domain
infrastructure -> application ports + domain
composition root -> all layers
```

domain 不得引用 application、infrastructure、interface、Tauri 或 reqwest。application 不得引用 infrastructure 或 interface。infrastructure 不得引用 interface。

### Scenario: HTTP 订阅探测

- **GIVEN** application 定义 `SubscriptionGateway` port
- **WHEN** Tauri command 执行订阅探测用例
- **THEN** composition/interface 注入 reqwest adapter
- **AND** 用例只通过 port 获取响应内容
- **AND** 解析和去重仍由 source domain 完成

## Added Requirement: 前端功能切片

React 前端 SHALL 以 `app / features / shared` 组织：应用装配只组合 feature，feature 内聚状态和 UI，跨 feature DTO 与 IPC adapter 位于 shared。

### Scenario: 新增来源类型

- **WHEN** 接入本地文件来源
- **THEN** 入口进入 source-inspection feature
- **AND** IPC 调用进入 shared API
- **AND** App shell 不实现解析或网络策略
