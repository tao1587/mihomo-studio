# Delta: 验证与导出

## ADDED Requirements

### Requirement: 静态隐私不变量

严格隐私 YAML 在序列化前与回读后 SHALL 验证：

- `dns.enable=true`、`enhanced-mode=fake-ip`、`respect-rules=true`；
- 上游 nameserver 明确绑定到代理策略；
- TUN 启用 DNS hijack、auto route 与 strict route；
- IPv6 在未验证闭环时关闭；
- rules、provider 更新和辅助网络连接中不存在未经批准的 `DIRECT` 路径；
- 节点域名不存在无保护 bootstrap。

任一检查失败 SHALL 阻止导出和 last-known-good 覆盖。

### Requirement: DNS 与出口证据分层

“DNS 路径已验证”和“最终出口已验证” SHALL 作为不同运行态证据记录；Mihomo 能加载配置不得推出二者成立。
