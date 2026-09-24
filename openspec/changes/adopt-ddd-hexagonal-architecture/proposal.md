# Proposal: 采用务实 DDD 与六边形架构

状态：`archived`

## 问题

当前实现规模不大，但边界已经开始混合：

- `catalog.rs` 同时承担领域模型、内嵌存储和 IPC；
- `profile.rs` 同时承担 DTO、catalog 获取、校验、领域决策和 IPC；
- `source/fetch.rs` 同时承担网络适配、URL 策略和订阅探测用例；
- `source/mod.rs` 既是模块入口又是 Tauri 接口；
- `App.tsx` 同时承担应用装配、IPC、状态、全部页面区块和展示逻辑；
- 浏览器预览与 Rust 草案存在双实现，但缺少明确的 adapter 边界。

继续在这些文件中叠加 resolver、缓存、Rule IR、导出和 sidecar，会形成跨层依赖和难以隔离测试的事务脚本。

## 目标

采用适合当前体量的 DDD + Hexagonal Architecture，而不是堆叠抽象：

```text
interface -> application -> domain
                    ^
                    |
             infrastructure
                 implements
             application ports
```

- `domain`：实体、值对象、领域错误、不变量和纯领域服务；
- `application`：用例编排，只依赖 domain 与 ports；
- `application/ports`：外部能力的抽象契约；
- `infrastructure`：HTTP、内嵌 JSON、未来文件/Keychain/Mihomo 适配器；
- `interface`：Tauri IPC DTO 边界和错误映射；
- `lib.rs`：唯一 composition root；
- React：采用 `app / features / shared` 功能切片，页面组件不直接散落 IPC 细节。

## 非目标

- 本次不增加 YAML 编译、缓存、resolver 或导出能力；
- 不改变 IPC 命令名和 JSON 字段；
- 不改变 UI 文案、默认规则、抓取限制和节点指纹；
- 不引入 DI 容器、event bus、CQRS 或数据库 repository；
- 不把每个数据结构机械包装成 entity/value object。

## 架构决策

1. bounded context 仍为 `catalog`、`profile`、`source`；layer 是依赖方向，context 是业务边界；
2. domain 不引用 Tauri、reqwest、React 或文件系统；
3. application port 使用小接口，只为当前真实外部依赖建模；
4. Tauri command 位于 `interface/ipc`，并把 typed error 转换为稳定安全字符串；
5. embedded catalog 与 reqwest subscription gateway 是 infrastructure adapters；
6. 纯 parser 和 profile policy 留在 domain，并在同层测试；
7. 前端 IPC 统一进入 `shared/api`，feature hooks 负责编排状态，section 组件负责展示。

## 兼容与风险

- Rust 文件移动可能影响测试模块路径和可见性；全量 `cargo test` 验证；
- `include_str!` 相对路径随目录变化；使用以新 adapter 文件为基准的路径并由 catalog 测试验证；
- async port 采用 boxed future，避免新增宏依赖；
- 浏览器预览仍保持本地 adapter，但由 shared API 明确区分 Tauri 与 preview；
- 当前目录没有 `.git`，已在 `/tmp/mihomo-studio-before-ddd` 保存本次迁移前副本。

## 验收

- 四层依赖方向可由脚本静态检查；
- `lib.rs` 只声明层并注册 IPC；
- 4 个 IPC 命令名、请求/响应 JSON 和错误文案保持兼容；
- 现有 Rust/前端测试全部通过；
- frontend build 与 `cargo check` 通过；
- Spec index、Implementation Map 与实际路径一致；
- 不产生真实 secret、提交、tag 或发布。

## 完成证据

- Rust 已迁移为 4 层、3 个 bounded context，并通过 ports/`ApplicationServices` 注入 adapters；
- React 已迁移为 `app / features / shared`；
- `npm run check:spec` 同时执行 Spec drift 与架构依赖门禁；
- 前端 6 项测试、Rust 23 项常规测试与 1 项 loopback ignored 集成测试通过；
- frontend build、`cargo fmt --check`、`cargo check`、Clippy `-D warnings` 通过；
- delta 已并入当前 `system-architecture` 及相关能力 Specs。
