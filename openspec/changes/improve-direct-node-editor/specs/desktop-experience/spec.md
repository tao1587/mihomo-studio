# Delta: 桌面体验

## ADDED Requirements

### Requirement: 多节点输入可读性

直接节点输入区 SHALL 使用完整内容宽度和多行编辑高度。编辑器 SHALL 保留显式换行，不得用软换行混淆 URI 边界，并 SHALL 显示非空输入行数量。

#### Scenario: 粘贴多个节点 URI

- **WHEN** 用户在直接节点输入区粘贴多个以换行分隔的 URI
- **THEN** 每个 URI 保持在独立视觉行
- **AND** 长 URI 可横向滚动，编辑区可纵向调整
- **AND** 界面显示忽略空行后的输入行数量
