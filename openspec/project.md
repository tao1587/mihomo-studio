# Mihomo Studio OpenSpec 入口

本目录是 Mihomo Studio 的**当前能力真相层**和 AI 阅读路由入口。任何代码阅读、设计或修改都先从本文件开始，再通过能力索引进入最小必要源码范围。

## 1. 产品边界

Mihomo Studio 是本地优先的 Mihomo 配置编译器：接收节点来源、规则来源和策略意图，最终生成可审计、可验证的 Mihomo 配置。它不是代理客户端，不接管系统代理或 TUN，也不把静态配置结果表述成真实网络出口结果。

当前版本为 `0.1.0`，实现阶段为 **Phase 2：Mihomo YAML 与 VLESS URI/Base64 URI 的严格隐私 YAML 编译路径已接通，独立“节点转换”功能可把 VLESS 或单 Peer WireGuard 客户端文本转换为 Clash/Mihomo `proxies:` 片段，单个 IP 服务器节点还可填入去敏内置模版并复制完整 YAML；域名节点可消费用户确认的 IPv4 bootstrap 映射，其他协议/provider resolver、绑定 sidecar 和应用内文件导出尚未接通**。

## 2. Spec-first 阅读协议

每次任务按以下顺序执行：

1. 运行 `git status --short --branch`、`git diff --stat`、`git diff --cached --stat`；若目录没有 Git 元数据，记录该事实，不初始化仓库、不覆盖文件；
2. 阅读本文件与 [`spec-index.json`](./spec-index.json)；
3. 选择一个主能力 Spec；只有跨能力任务才读取额外 Spec；
4. 先读 Spec 的 `Implementation Map` 所列文件，不做全仓库搜索；
5. 仅在 Spec 漂移、入口缺失、跨能力依赖或安全证据不足时扩大搜索，并在变更中补回缺失映射；
6. 行为变化先写 `openspec/changes/<change-id>/`，确认后同步能力 Spec 与实现；
7. 运行能力 Spec 列出的验证，再运行 `npm run check:spec`。

## 3. 真相层级

从高到低：

1. `openspec/specs/*/spec.md`：当前能力、约束、契约和实现映射；
2. 当前源码与测试：实现证据；若与 Spec 冲突，先记录 drift，不静默选择任一方；
3. `openspec/changes/`：尚未并入当前能力的行为变化；
4. `openspec/roadmap.md`：跨能力待办，不代表已经实现；
5. `docs/`：产品研究、决策背景、生态快照和旧阶段说明，不作为当前实现的并行真相。

Spec 用来限定搜索范围，不替代源码、构建产物或运行态验证。

## 4. 能力路由

| 能力 | 何时进入 | Spec |
|---|---|---|
| 系统架构 | 模块边界、Tauri 组合、领域分层、跨能力状态 | [`system-architecture`](./specs/system-architecture/spec.md) |
| 节点来源接入 | 订阅请求、URI/Base64/YAML 识别、节点指纹 | [`source-ingestion`](./specs/source-ingestion/spec.md) |
| 规则目录 | 规则源 catalog、来源身份、family 互斥、MRS 边界 | [`rule-catalog`](./specs/rule-catalog/spec.md) |
| 配置草案与编译 | ProfileDraft、策略组、规则顺序、Node/Rule IR | [`profile-compilation`](./specs/profile-compilation/spec.md) |
| 桌面体验 | React 向导、简单/完全模式、状态文案 | [`desktop-experience`](./specs/desktop-experience/spec.md) |
| 验证与导出 | YAML、Mihomo sidecar、文件导出、Clash Verge 证据 | [`validation-export`](./specs/validation-export/spec.md) |
| 安全与隐私 | 秘密、网络边界、日志、缓存和报告脱敏 | [`security-privacy`](./specs/security-privacy/spec.md) |
| Spec 治理 | 文档层级、变更流程、漂移检查和验收证据 | [`spec-governance`](./specs/spec-governance/spec.md) |

机器可读的路径、命令和验证入口见 [`spec-index.json`](./spec-index.json)。

## 5. 不可破坏的项目级不变量

- `src-tauri/src/lib.rs` 只负责 Tauri 组合、生命周期和 IPC 注册；业务逻辑位于领域模块；
- 订阅 URL、UUID、密码和 token 都是秘密，不进入日志、错误详情、测试夹具或生成报告；
- canonical URL 标识来源身份，镜像只产生 effective transport URL；
- 相同 matcher 指向不同策略是冲突，不是可静默删除的重复；
- 最终配置只能有一个 `MATCH`，且它必须位于最后；
- “已识别”“草案就绪”“已生成”“Mihomo 已验证”“已导出”“Clash Verge 已加载”“规则命中/出口已验证”是七个独立状态；
- 未收到明确指令时，不提交、不推送、不打 tag、不发布。

## 6. 验证入口

```bash
npm run check:spec
npm test
npm run build
cd src-tauri && cargo fmt --check && cargo test && cargo check
```

测试通过只证明对应源码门禁，不证明导出文件已被 Clash Verge 加载，也不证明规则命中或最终公网出口。
