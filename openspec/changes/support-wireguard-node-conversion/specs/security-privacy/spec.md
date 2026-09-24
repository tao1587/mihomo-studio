# Delta: 安全与隐私

## MODIFIED Requirements

### Requirement: Secret lifecycle

独立节点转换输入和输出 SHALL 只存在于当前 React 状态、当前 IPC 参数/结果、Rust 调用内存和用户主动复制的剪贴板。VLESS UUID、WireGuard 私钥/公钥、Endpoint 和生成 YAML 仍属于 secrets。

用户主动转换成功后，独立转换页 MAY 按个人本地工具的明确产品边界直接显示完整敏感 YAML；这只放宽当前页面的会话内披露，不得把秘密写入 live-region、日志、错误、遥测、测试报告、缓存、文件或持久化，也不得自动写剪贴板。切离页面或编辑输入 SHALL 销毁/清除过期转换状态。

#### Scenario: 预览 WireGuard 节点片段

- **WHEN** 用户主动把 WireGuard 客户端文本转换为 `proxies:` YAML
- **THEN** 完整密钥保留在当前页面直接可见的敏感结果中
- **AND** 无秘密摘要、错误和辅助技术状态播报不包含密钥、Endpoint 或其他输入值
- **AND** 只有用户主动点击复制时结果才进入剪贴板。

### Requirement: 测试夹具

WireGuard 转换测试 SHALL 仅使用保留地址、`.invalid` 域名和明显占位的私钥、公钥。用户提供的真实配置、节点或设备标识不得进入 fixture、snapshot、golden、测试输出或生成报告。
