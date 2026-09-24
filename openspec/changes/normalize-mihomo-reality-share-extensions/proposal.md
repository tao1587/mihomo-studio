# Proposal: 显式归一化 Mihomo 不支持的 REALITY 分享扩展

状态：`archived`

## 问题

部分标准 VLESS REALITY 分享链接携带 `spx`（spider path）和 `pqv`（ML-DSA-65 verify key）。当前严格编译器把非默认 `spx` 或任何未映射参数统一报告为“尚未安全映射”，因此用户无法区分真正未知字段与目标 Mihomo 内核已知不支持的 Xray 扩展。

当前 Mihomo Alpha 的 `RealityOptions` 只接受 `public-key`、`short-id` 和 `support-x25519mlkem768`；其 VLESS 分享链接转换器也不把 `spx`/`pqv`写入目标 YAML。Mihomo Studio 需要遵守该目标能力边界，但不得像上游转换器一样静默丢弃。

## 目标

1. 在 `security=reality` 时识别并消费 `spx` 与 `pqv`，不再把它们误报为未知分享参数；
2. 不把这两个字段或其值写入 Mihomo YAML，因为目标内核没有等价字段；
3. 在无秘密生成报告中按受影响节点数量明确提示 REALITY 目标兼容降级；
4. DNS、bootstrap、Provider、DIRECT、IPv6 和未知分享字段继续保持原有 fail-closed 边界；
5. `spx`/`pqv` 出现在非 REALITY 节点时继续拒绝，不将目标特例扩大为通用忽略机制。

## 验收场景

### REALITY 链接包含目标不支持扩展

- **GIVEN** VLESS URI 使用 `security=reality`、有效 `pbk`，并携带非空 `spx` 或 `pqv`
- **WHEN** 编译为 Mihomo Node IR
- **THEN** 编译器按 Mihomo 目标边界省略这些字段
- **AND** 继续保留 `public-key`、`short-id`、TLS、transport 与严格隐私约束
- **AND** 报告只写受影响节点数和参数类别，不包含参数值、节点名称、server、UUID 或订阅信息。

### 未知参数仍然失败

- **GIVEN** 分享链接包含 `spx`/`pqv` 之外的未知参数
- **WHEN** 编译为 Node IR
- **THEN** 编译失败
- **AND** 错误不回显未知参数值或完整 URI。

## 安全与回滚

- `strictPrivacy=true` 继续只陈述 DNS、路由、bootstrap 和 Provider 出口不变量；报告同时陈述目标协议能力降级，避免把目标兼容误写为完全等价转换。
- 测试只使用保留地址和 `PLACEHOLDER_*` 数据。
- 回滚副本：`/tmp/mihomo-studio-before-reality-compat-20260815T221024`。
- 当前目录没有 Git 元数据；不初始化仓库，不提交或推送。
