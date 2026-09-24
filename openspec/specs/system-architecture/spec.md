# Capability: 系统架构

| 字段 | 值 |
|---|---|
| ID | `system-architecture` |
| 状态 | `partial` |
| 当前实现 | Tauri 2 + Rust 2021 + React 19 + TypeScript |
| 边界 | 本地配置编译器，不是代理客户端 |

## Purpose

定义稳定的分层、依赖方向、组合入口和跨能力状态，确保新增功能进入领域模块，而不是在 UI 或 `lib.rs` 中形成隐式业务逻辑。

## Current State

- Rust 已按 `domain / application / infrastructure / interface` 四层组织；
- `catalog`、`profile`、`source` 是三个 bounded context，domain 中保存模型与不变量；
- application 用例只依赖 domain 和 `application/ports.rs`；
- embedded catalog 与 reqwest 抓取器是 infrastructure adapter；
- Tauri IPC 位于 `interface/ipc`，`lib.rs` 负责注入 `ApplicationServices` 和注册 6 个命令；独立节点转换使用协议中性的 `convert_node_text`；
- React 已按 `app / features / shared` 切片，浏览器开发模式使用明确的本地草案 adapter；
- Node/Rule/Provider/Group IR 与纯 YAML compiler 已位于 `domain/compile`；缓存、绑定 sidecar 和导出 adapter 仍未落地。

## Requirements

### Requirement: 分层与依赖方向

系统 SHALL 保持以下单向依赖：

```text
interface -> application -> domain
infrastructure -> application ports + domain
composition root -> all layers
```

domain 不得依赖 application、infrastructure、interface、Tauri 或 reqwest；application 不得依赖 infrastructure、interface、Tauri 或 reqwest；interface 通过 `ApplicationServices` 使用 port，不直接构造 infrastructure adapter。

#### Scenario: 新增规则下载

- **WHEN** 实现远程规则下载与缓存
- **THEN** 下载策略和 canonical/effective URL 逻辑进入领域或适配器模块
- **AND** `lib.rs` 仅增加模块组合和命令注册

### Requirement: Tauri 组合根

`src-tauri/src/lib.rs` SHALL 只包含模块声明、adapter 装配、依赖注入、Tauri Builder、IPC 注册和应用生命周期。解析、校验、下载、编译、缓存、导出和秘密处理 SHALL 位于独立模块。

#### Scenario: 审查组合根

- **WHEN** `npm run check:spec` 检查 Rust 入口
- **THEN** 它识别 `application`、`domain`、`infrastructure`、`interface` 四个已登记层
- **AND** 识别 6 个已登记 IPC 命令

### Requirement: 前后端契约

IPC 请求和响应 SHALL 使用可序列化 DTO；字段使用 camelCase 边界。Rust 与 TypeScript 的命令名称和核心状态语义必须同步。

#### Scenario: IPC 漂移

- **WHEN** Rust 注册命令被添加、删除或重命名
- **THEN** `spec-index.json` 和相关能力 Spec 必须在同一变更中更新
- **AND** 漂移检查在未同步时失败

### Requirement: 状态不可合并

系统 SHALL 分别表达以下状态：

1. 来源已识别；
2. 草案可进入编译；
3. YAML 已生成；
4. 指定 Mihomo 已验证；
5. 文件已导出；
6. Clash Verge 已加载；
7. 实际规则命中与最终出口已验证。

任何较早状态不得推断较晚状态。

#### Scenario: 草案构建成功

- **GIVEN** 节点数量与规则来源满足草案条件
- **WHEN** `build_profile_draft` 返回 `readyForCompilation=true`
- **THEN** UI 只能显示“可进入编译”
- **AND** 不得显示“配置已生成”或“Mihomo 已验证”

### Requirement: 跨平台核心

配置语义 SHALL 由共享 Rust 领域核心实现。平台差异只存在于凭据库、文件权限、进程和应用集成适配器，且必须有显式降级状态。

### Requirement: 前端功能切片

React SHALL 以 `app / features / shared` 组织：app 只装配页面与 feature controllers；feature 内聚自己的 model/UI；shared 保存跨 feature contract、IPC adapter 和无业务归属的 UI/工具。

shared 不得依赖 feature/app，feature 不得依赖 app，也不得跨 feature 直接引用内部实现。

#### Scenario: 新增本地文件来源

- **WHEN** 接入本地文件来源
- **THEN** 输入和展示进入 source-inspection feature
- **AND** IPC 调用进入 shared API adapter
- **AND** App shell 不实现解析、文件或网络策略

## Implementation Map

| 路径 | 责任 |
|---|---|
| `src-tauri/src/lib.rs` | Composition root、adapter 注入与 IPC 注册 |
| `src-tauri/src/domain/` | bounded contexts、值对象、不变量与纯领域服务 |
| `src-tauri/src/application/` | 用例、ports 与 ApplicationServices |
| `src-tauri/src/infrastructure/` | embedded catalog、reqwest 等 outbound adapters |
| `src-tauri/src/interface/ipc/` | Tauri inbound adapter 与安全错误映射 |
| `src/app/` | React 应用装配与 workspace 状态编排 |
| `src/features/` | 来源、规则构建和预览功能切片 |
| `src/shared/` | IPC adapter、跨功能 contracts、UI 和纯展示工具 |

## Verification

```bash
npm run check:spec
npm test
npm run build
cd src-tauri && cargo fmt --check && cargo test && cargo check
```

## Known Gaps

- Rust domain 模型当前直接承担 serde contract；DTO 复杂化时应在 application contract 中增加显式映射；
- 浏览器预览与 Rust 草案仍存在双实现，新增语义必须增加一致性测试或共享 schema；
- `useStudioWorkspace` 是当前前端页面级编排器，新增持久化/路由后应拆成更小的 feature controllers。
