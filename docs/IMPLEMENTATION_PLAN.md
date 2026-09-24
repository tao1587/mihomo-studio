# 实施计划

## 总原则

- Phase 1 已启动；每一阶段都保持可构建、可测试和事实边界清晰；
- 用户确认关键决策后再推进下一阶段；
- 每一阶段都先定义领域模型和验收用例，再做 UI；
- 不把 NAS 脚本直接搬进 Tauri；
- 配置生成、Mihomo 验证、Clash Verge 加载、实际规则命中分别报告。

## Phase 0：设计冻结与样本夹具（进行中）

- [x] 确认项目名、目录和 UI 技术栈；
- [x] 确认简单模式默认规则包；
- [x] 完成开源规则生态调研和首版 catalog；
- [x] 确认 Claude 规则默认开关；
- [ ] 收集脱敏夹具：
  - VLESS Reality；
  - VLESS WS/gRPC；
  - VMess；
  - Trojan；
  - SS；
  - Hysteria2/TUIC/WireGuard；
  - YAML 订阅；
  - Base64 URI 订阅；
  - 根据 User-Agent 返回不同格式的订阅；
  - 错误页、登录页、空订阅、超大响应；
- [ ] 为期望输出建立 golden files。

验收：所有样本不含真实 token、UUID、密码或服务地址。

## Phase 1：Tauri 骨架和领域模型（2 天）

- [x] 初始化 Tauri 2 + React + TypeScript + Vite；
- [x] 创建 Rust catalog/profile 模块边界；
- [x] 定义 RuleSource、CatalogRuleSet 和 ProfileDraft；
- [x] 建立首批命令 DTO 和错误分类；
- [ ] 定义 Node IR、Rule IR、PolicyGroup、CompileReport；
- [ ] 建立脱敏日志和 secret reference；
- [x] 建立前端/Rust 基础单测；
- [ ] 建立 macOS、Windows、Linux CI。

验收：空项目可以在 macOS/Windows/Linux 构建，领域模型可序列化并有 schema version。

## Phase 2：节点来源与订阅识别（进行中）

- [x] Rust HTTP fetcher；
- [x] 系统代理与直连抓取路径；
- [x] User-Agent 探测；
- [x] YAML/URI/Base64 识别；
- [x] 直接节点解析；
- [x] 来源安全标签和协议名称规范化；
- [x] Node 精确指纹与单来源去重；
- [ ] 跨来源 provenance 合并；
- [ ] last-known-good 缓存；
- [ ] secret reference 与操作系统凭据库存储。

验收：多个 YAML 来源真正合并；节点名称改变不破坏稳定 ID；秘密不进入日志。

## Phase 3：短生命周期 Mihomo 解析/验证适配器（2～3 天）

- sidecar 版本和 hash 清单；
- 临时 HomeDir；
- 随机 loopback Controller 与 secret；
- proxy-provider 加载和节点枚举；
- 进程超时、终止、清理；
- 配置静态验证；
- 不启用节点健康检查。

验收：本地解析器未知但 Mihomo 可识别的 URI/Base64/YAML 订阅可被标准化；sidecar 退出后无常驻端口和秘密文件。

## Phase 4：规则目录、GitHub 镜像和缓存（2～3 天）

- [x] RuleSource Registry schema v1；
- [x] ACL4SSR 仓库和首批具体 rule-set；
- [x] MetaCubeX 仓库和 opaque MRS 标记；
- [x] blackmatrix7 服务目录入口；
- [x] 热门开源规则 family 与互斥/重叠元数据；
- [ ] 全局/单源 GitHub 镜像 fetcher；
- [x] catalog 中定义 canonical identity；[ ] 实现 effective transport；
- [ ] ETag/hash/last-known-good；
- [x] UI 展示许可证、star/fork 快照和维护状态。

验收：镜像切换不创建重复来源；更新失败不覆盖有效缓存；非 GitHub 资产不会被错误拼接 GitHub 镜像。

## Phase 5：规则 IR、去重与冲突引擎（3～5 天）

- 多格式 parser；
- exact dedup；
- 域名后缀包含；
- CIDR 包含；
- 不同 target 冲突；
- 优先级编译；
- provenance 报告；
- `MATCH` 不变量；
- 大规则集性能测试。

验收：每一条删除都能解释；keyword 关系默认不自动删除；不同策略的精确例外被保留。

## Phase 6：策略组编译与简单/完全模式（3～4 天）

- [x] Simple Recipe 结构草案；
- [x] Full Recipe 结构草案；
- 动态地区识别；
- 组图循环/空组/引用检测；
- 服务策略绑定；
- Provider/快照/自动输出；
- [x] UI 三步向导骨架；
- 完全模式卡片和优先级编辑。

验收：简单模式 3 步导出；完全模式可解释所有生成组和规则来源。

## Phase 7：Claude 规则包（1～2 天）

- 从 Claude Purge 当前受管清单生成版本化数据；
- 进程规则；
- 第一方域名规则；
- 可选 UDP fail-closed；
- 可选精确遥测拦截；
- 平台能力降级提示；
- 与宽泛规则的顺序测试。

验收：Claude 规则位于 CN/MATCH 之前；共享基础设施不被宽泛域名拦截；遥测开关独立。

## Phase 8：导出、回滚和跨平台发布（3～5 天）

- 原子导出和 last-known-good；
- 私有文件权限；
- diff 预览；
- Clash Verge 一次导入说明；
- 安装包签名/公证/校验；
- macOS、Windows、Linux E2E；
- 第三方许可证清单。

验收：安装包、源码实现、导出文件、Mihomo 验证和 Clash Verge 运行态分别有证据。

## 后续 Phase 9：托管本地订阅

首版稳定后再考虑：

- loopback 本地订阅服务；
- 定时刷新并原子发布；
- Clash Verge 只需订阅一次本地 URL；
- 随机 token、loopback-only、状态页；
- 失败时继续提供 last-known-good；
- 明确的启动项开关和卸载清理。

这能进一步减少手动导入，但会引入常驻服务、端口、启动项和生命周期复杂度，不放进首版 MVP。

## 工期预估

- 可用 MVP：约 10～15 个工作日；
- 完整跨平台体验、安装包、规则目录和稳定错误处理：约 4～6 周；
- 本地托管订阅与自动同步：在 MVP 之后追加约 1～2 周。

工期主要风险不在 UI，而在订阅兼容、规则冲突语义、sidecar 跨平台分发和真实样本覆盖。
