# Capability: 规则目录

| 字段 | 值 |
|---|---|
| ID | `rule-catalog` |
| 状态 | `partial` |
| IPC | `get_rule_catalog` |
| Catalog schema | `1` |

## Purpose

以稳定身份描述可选规则生态、许可证、来源族、互斥关系、规则顺序与审计边界，避免把下载镜像、热度快照或最终 URL 当作来源身份。

## Current State

- 内嵌 catalog 当前有 10 个 repository source 和 12 个具体 rule set；
- 简单模式默认 source 为 `acl4ssr`；
- 简单模式默认 rule set 有 9 个，其中唯一 `builtin.match` 的 order 为 1000；
- Rust 与前端测试覆盖 ID 唯一、HTTPS repository、默认广告源、MRS opaque 和历史来源缺席；
- 规则下载、effective URL、缓存、内容 hash、resolved commit 和逐条 Rule IR 尚未实现。

热度、许可证和维护状态是 `researchedAt` 对应的研究快照，使用或发布前必须复核。

## Requirements

### Requirement: Canonical identity

规则来源 SHALL 由稳定 source/rule-set ID、repository、ref、path 和 canonical URL 标识。GitHub 镜像只产生 effective transport URL，不得改变 cache key、provenance 或来源身份。

#### Scenario: 更换 GitHub 镜像

- **GIVEN** canonical URL 不变
- **WHEN** 用户更换镜像前缀
- **THEN** effective URL 可以变化
- **AND** 来源 ID、canonical URL 和 provenance 保持不变

### Requirement: Catalog schema

每个 source SHALL 描述模式可用性、默认启用模式、优先级、策略目标、维护状态、选择类型、审计类型、许可证和可选互斥组。每个 rule set SHALL 描述 family、canonical location、behavior、format、target、order、mirror kind 和归属要求。

#### Scenario: 新增可选择来源

- **WHEN** catalog 增加一个 source
- **THEN** ID 在全部 source 中唯一
- **AND** repository 使用 HTTPS
- **AND** 它的 selection kind、license、audit boundary 和模式可用性均已声明

### Requirement: Family exclusivity

同一 exclusive group 中只允许选择一个 source。冲突选择 SHALL 被拒绝或要求显式解决，不得按导入顺序静默覆盖。

#### Scenario: 两个核心 routing family

- **WHEN** 请求同时选择 `acl4ssr` 和 `loyalsoldier-clash-rules`
- **THEN** 草案校验返回 `routing.core` 互斥错误

### Requirement: Audit boundary

text/yaml 输入可声明逐条审计；MRS SHALL 标记 `opaque`，在没有同源可审计文本或绑定 Mihomo 转换证据时只能做 provider/source/hash 级去重。

#### Scenario: 选择 MRS 来源

- **WHEN** 用户选择 opaque MRS source
- **THEN** 草案显示 provider 级去重警告
- **AND** 报告不得声称已完成内容级规则去重

### Requirement: 默认规则不变量

简单模式默认集 SHALL：

- 只有一个 `ads.primary`；
- 只有一个最终 `MATCH`；
- 最终 `MATCH` 的 order 为 1000；
- 服务专用规则位于宽泛地理规则和 `MATCH` 之前。

### Requirement: 引用型上游

`selectionKind=upstream-data` 的项目只作为数据血缘登记，不得作为普通可选规则源进入草案。

## Implementation Map

| 路径 | 责任 |
|---|---|
| `src/data/rule-catalog.json` | Catalog schema v1 数据 |
| `src-tauri/src/domain/catalog/` | Catalog 模型、typed error 和结构校验 |
| `src-tauri/src/application/catalog.rs` | 获取 catalog 用例 |
| `src-tauri/src/application/ports.rs` | CatalogRepository port |
| `src-tauri/src/infrastructure/catalog/embedded.rs` | 内嵌 JSON repository adapter 与不变量测试 |
| `src-tauri/src/interface/ipc/catalog.rs` | Tauri catalog command |
| `src/features/profile-builder/model/catalog.ts` | 模式可见性、默认选择和浏览器草案纯函数 |
| `src/features/profile-builder/model/catalog.test.ts` | 前端 catalog/草案不变量 |
| `src/features/profile-builder/ui/` | 规则来源展示与选择 |

## Verification

```bash
npm run check:spec
npm test -- --run src/features/profile-builder/model/catalog.test.ts
cd src-tauri && cargo test catalog::
```

涉及远程研究字段时，还需单独复核上游 repository、许可证、维护状态和 resolved commit；静态 catalog 测试不证明远程来源当前可用。
