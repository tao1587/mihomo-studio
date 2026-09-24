# Delta: 系统架构

## MODIFIED Requirements

### Requirement: 前后端契约

独立节点转换 SHALL 使用协议中性的 `convert_node_text` IPC、`ConvertNodeTextRequest` 和 `ConvertNodeTextResult`。Rust 注册命令、application/domain contract、TypeScript adapter/contracts 与 `spec-index.json` SHALL 在同一变更中取代旧 VLESS 专用命名，字段继续使用 camelCase 边界。

#### Scenario: 节点转换 IPC 迁移

- **WHEN** 应用构建并运行 Spec 漂移检查
- **THEN** 只注册并索引 `convert_node_text`
- **AND** 前端不再调用 `convert_vless_node_text`。
