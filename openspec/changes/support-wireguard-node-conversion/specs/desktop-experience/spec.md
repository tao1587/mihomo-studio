# Delta: 桌面体验

## MODIFIED Requirements

### Requirement: 独立节点转换功能入口

应用 SHALL 保留顶栏“节点转换”一级入口和独立页面。输入提示 SHALL 明确支持单条/多行/Base64 VLESS，以及一份 WireGuard `[Interface]` / `[Peer]` 客户端配置；不得要求用户先选择协议。

转换结果 SHALL 显示输出节点数和去重数，并明确标记为敏感 `proxies:` 片段。用户主动转换成功后，完整 YAML SHALL 在当前独立页面直接可见且提供显式复制操作，不得再放入默认关闭或可再次隐藏的 disclosure。该 YAML 不得位于 live-region；只有无秘密成功摘要可作为状态更新播报。

该常显规则只适用于独立节点转换结果。完整配置 YAML、订阅 URL 和其他秘密界面仍遵守既有折叠或遮罩规则。

#### Scenario: WireGuard 转换结果

- **WHEN** 用户粘贴有效 WireGuard 客户端文本并主动转换
- **THEN** 页面显示一个已生成节点的无秘密摘要
- **AND** 完整敏感 YAML 无需再次展开即可见
- **AND** 页面不声称已导入、已连接或已验证出口。
