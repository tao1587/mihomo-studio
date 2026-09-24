# Mihomo Studio 跨能力路线图

路线图描述尚未完成的跨能力工作；完成项应移动到对应能力 Spec 的 `Current State`，而不是长期保留双份状态。

## Now — Phase 2 收口

- 在已接通的 IPv4 literal Mihomo YAML 与 VLESS URI Node IR 上继续实现 VMess/Trojan/SS 等协议、provider resolver 与完整跨来源 provenance；
- 为订阅来源增加 secret reference 与操作系统凭据库存储；
- 增加 last-known-good 内容寻址缓存；
- 为本地 parser 未覆盖的 provider 接入短生命周期 Mihomo resolver；
- 建立脱敏 fixture 与 golden output，覆盖正常、错误页、超大响应和 UA 差异。

## Next — 规则编译

- 实现 canonical URL 到 effective URL 的 GitHub transport 重写；
- 下载、校验并缓存规则源；
- 扩展现有严格 Rule/Provider IR，接通规则内容下载、精确去重、保守语义去重与冲突报告；
- 在已实现的策略组图、引用、循环、唯一 MATCH 和 YAML 回读上增加更多协议/模式 golden；
- 将当前“草案 + 严格 YAML”两阶段补齐为带完整 provenance 的编译结果。

## Then — 验证与交付

- 固定 Mihomo sidecar 版本与 hash；
- 配置解析回读和 Mihomo 静态加载验证；
- 原子导出、私有权限、last-known-good 和 diff 预览；
- macOS、Windows、Linux CI 与安装包验证；
- 记录 Clash Verge 加载、规则命中和最终出口的独立运行态证据。

## Later — 本地托管订阅

- loopback-only 本地订阅服务；
- 随机 token、原子刷新和 last-known-good；
- 明确的启动项、关闭、卸载和残留清理语义。

此能力不进入首版 MVP；启用前必须单独提交 change proposal。
