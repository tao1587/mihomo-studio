# Delta: 节点来源接入

## MODIFIED Requirements

### Requirement: 独立节点片段转换

系统 SHALL 提供协议感知、独立于完整配置编译的节点片段转换。输入支持既有单条、多行或 Base64 VLESS URI，以及一份包含一个 `[Interface]` 和一个 `[Peer]` 的标准 WireGuard 客户端文本；输出为可合并进 Clash/Mihomo 配置的 `proxies:` YAML。

VLESS SHALL 继续复用严格编译器的权威字段映射、精确去重、稳定重名和显式兼容归一化。WireGuard SHALL 映射 `PrivateKey`、IPv4/可选 IPv6 `Address`、可选 `DNS`、`PublicKey`、`Endpoint`、`AllowedIPs` 与可选 `PersistentKeepalive`；`Address` 的 CIDR 前缀 SHALL 原样保留其规范化语义，DNS 存在时 SHALL 同时启用节点级 `remote-dns-resolve`。没有名称的配置 SHALL 使用稳定非秘密名称 `WireGuard 1`。

WireGuard 缺少必要字段、字段重复、值格式无效、包含多个 Peer 或包含未映射字段时 SHALL fail-closed，不返回部分 YAML。错误不得回显字段值、Endpoint、地址或密钥。

转换结果 SHALL 报告输出节点数、重复数、目标兼容归一化数和无秘密警告。转换器支持 WireGuard 只代表独立片段转换能力，不得自动把 WireGuard INI 标记为完整配置编译就绪。

#### Scenario: 单 Peer WireGuard 客户端配置

- **GIVEN** 一份包含私钥、IPv4 Address、DNS、单个 Peer 公钥、Endpoint、AllowedIPs 和 keepalive 的 WireGuard 客户端文本
- **WHEN** 调用 `convert_node_text`
- **THEN** 输出一个 `type: wireguard` 的 Mihomo 节点
- **AND** 目标节点使用 `peers` 完整写法并保留完整密钥到敏感 YAML。

#### Scenario: 多 Peer 拒绝

- **GIVEN** WireGuard 输入包含多个 `[Peer]`
- **WHEN** 调用转换
- **THEN** 转换以稳定类别全量失败
- **AND** 不返回部分 YAML或回显任一 Peer 内容。

## MODIFIED Contracts

`ConvertNodeTextRequest`：

- `content: String`：secret-bearing 输入，只在当前调用内存使用，不实现 Debug 输出。

`ConvertNodeTextResult`：

- `yaml`：敏感 `proxies:` YAML 片段，保留连接所需凭据；
- `nodeCount` / `duplicateNodeCount`；
- `compatibilityNormalizationCount`；
- `warnings[]`：只含数量与稳定兼容提示。

IPC：`convert_node_text` 取代 `convert_vless_node_text`。
