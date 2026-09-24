# Capability: Spec 治理

| 字段 | 值 |
|---|---|
| ID | `spec-governance` |
| 状态 | `implemented` |
| 入口 | `openspec/project.md` |

## Purpose

让 AI 和开发者以能力为单位读取和修改项目，减少全仓搜索与上下文爆炸，同时保证 Spec、源码契约和验证入口不会静默漂移。

## Requirements

### Requirement: 唯一当前真相层

当前行为 SHALL 只在 `openspec/specs/*/spec.md` 中维护。`docs/` 可以保存研究、决策背景和时间绑定快照，但不得作为并行的当前实现真相。

#### Scenario: 文档冲突

- **WHEN** 背景文档与能力 Spec 或源码不同
- **THEN** 记录为 Spec drift
- **AND** 通过当前源码和验收证据确定修正方向
- **AND** 不静默复制背景文档中的旧结论

### Requirement: 有界阅读

代码任务 SHALL 从 `openspec/project.md` 和 `spec-index.json` 开始，选择主能力后只读取其 Implementation Map。只有以下情况可以扩大搜索：

- Spec 中登记的入口缺失；
- 当前源码与 Spec 明确漂移；
- 任务跨越多个能力；
- 安全、隐私、许可证或运行态证据需要追踪来源到汇；
- 构建/测试失败指向未登记路径。

扩大后必须在同一任务中补充遗漏的映射或 gap。

### Requirement: 变更工件

改变用户可观察行为、契约、不变量、持久化格式、外部集成或安全边界前，SHALL 在 `openspec/changes/<change-id>/` 创建 proposal、tasks 和 delta Spec。纯拼写或不改变语义的文档修正不需要 change artifact。

#### Scenario: 新增规则缓存

- **WHEN** 计划增加规则下载与 last-known-good 缓存
- **THEN** 先写受影响的 `rule-catalog`、`security-privacy` 和 `validation-export` delta
- **AND** 定义迁移、失败回退和验收场景

### Requirement: Requirement 与 Scenario

每条规范性行为 SHALL 使用 `MUST/SHALL` 级别措辞，并至少为关键成功、失败或边界行为提供 Given/When/Then 场景。计划性愿望只能进入 roadmap 或标记为 planned，不得写成已实现事实。

### Requirement: Implementation Map

每个能力 Spec SHALL 列出最小源码入口、DTO/IPC、验证命令和已知缺口。新增、删除或移动入口时，Spec 与 `spec-index.json` 必须同步。

### Requirement: Drift gate

`scripts/check-spec-drift.mjs` SHALL 至少验证：

- 三个版本来源一致；
- 索引中的能力 Spec 和实现路径存在；
- Rust 模块与 IPC 注册符合索引；
- 前端调用的 IPC 已登记；
- catalog schema、数量、简单模式默认源和最终 MATCH 不变量符合索引；
- Spec/README 的本地 Markdown 链接有效。

`scripts/check-architecture.mjs` SHALL 同时验证 Rust 四层依赖和 React feature-slice 依赖，禁止旧路径重新出现。

#### Scenario: 新增 IPC 未登记

- **WHEN** `lib.rs` 注册新命令但 `spec-index.json` 未更新
- **THEN** `npm run check:spec` 失败并列出差异

### Requirement: 验证证据分层

报告 SHALL 分开列出：

- Spec drift；
- 源码格式/单元测试/静态构建；
- 生成配置；
- Mihomo 验证；
- 文件导出；
- Clash Verge 加载；
- 规则命中和最终出口；
- CI、安装包和发布状态。

未执行的层级明确写“未验证”，不得由前一层推断。

### Requirement: 工作区保护

每次修改前 SHALL 运行三项 Git 基线命令。若目录不是 Git 仓库，SHALL 报告并继续保护现有文件；不得自行初始化、reset、清理、提交、推送、打 tag 或发布。

## Implementation Map

| 路径 | 责任 |
|---|---|
| `AGENTS.md` | 每次任务的强制工作协议 |
| `openspec/project.md` | 人类/AI 路由入口和项目级不变量 |
| `openspec/spec-index.json` | 能力、路径、IPC 和 drift expectation |
| `openspec/specs/*/spec.md` | 当前能力真相 |
| `openspec/changes/README.md` | change artifact 生命周期 |
| `openspec/roadmap.md` | 跨能力未实现工作 |
| `docs/README.md` | 背景文档分类 |
| `scripts/check-spec-drift.mjs` | 自动漂移门禁 |
| `scripts/check-architecture.mjs` | Rust/React 架构依赖门禁 |

## Verification

```bash
npm run check:spec
```

漂移脚本是最低门禁，不替代人工审查 requirement 是否准确、场景是否覆盖失败路径、研究字段是否仍然有效。
