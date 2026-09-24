# Proposal: 归一化订阅导出的 VLESS 别名与无效传输字段

状态：`archived`

## 问题

真实订阅在 Mihomo User-Agent 下返回的 VLESS URI 同时携带等值 `sni`/`servername`，并在 `type=tcp` 节点上附带 gRPC `mode=multi`。当前编译器只识别 `sni`，且把 TCP 上除 `gun` 外的 `mode` 当作阻塞字段，因此即使 REALITY `pqv`/`spx` 已处理，第一节点仍以同一条泛化错误失败。

这些字段组合来自订阅导出器，不改变 Mihomo TCP 节点的连接语义：`servername` 是 `sni` 的目标字段别名，TCP 不消费 gRPC mode。官方 Mihomo VLESS 转换器同样读取 `sni` 并只在对应 transport 分支消费 transport option。

## 目标

1. 接受 `servername` 作为 `sni` 的分享别名；两者同时存在时只允许值一致；
2. `type=tcp`/`raw` 时允许并省略已知 gRPC exporter mode：`gun`、`multi`、`guna`；
3. 省略 TCP mode 时计入目标兼容归一化警告，不静默改变报告；
4. 冲突别名、未知 TCP mode、其他未知参数继续 fail-closed；
5. 错误、报告和测试不包含真实订阅、节点地址、UUID 或参数值。

## 验收场景

### 等值 servername 别名

- **GIVEN** VLESS URI 同时包含等值 `sni` 与 `servername`
- **WHEN** 构建 Mihomo Node IR
- **THEN** 只生成一个 `servername` 字段
- **AND** 不把别名残留为未知参数。

### TCP 节点携带 gRPC mode

- **GIVEN** `type=tcp` URI 携带 `mode=multi`
- **WHEN** 构建 Mihomo Node IR
- **THEN** 节点保持 `network: tcp`
- **AND** mode 不进入 YAML
- **AND** 报告提示发生目标兼容归一化。

### 冲突别名

- **GIVEN** `sni` 与 `servername` 值不同
- **WHEN** 构建 Node IR
- **THEN** 编译失败
- **AND** 错误不回显任一值。

## 回滚

- 修改前副本：`/tmp/mihomo-studio-before-provider-alias-fix-20260815T224623`。
- 当前目录没有 Git 元数据；不初始化仓库，不提交或推送。
