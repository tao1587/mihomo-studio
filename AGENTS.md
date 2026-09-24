# Mihomo Studio 工作协议

1. 每次修改前运行 `git status --short --branch`、`git diff --stat` 和 `git diff --cached --stat`，保护现有改动。
2. 所有代码阅读与修改从 `openspec/project.md` 和 `openspec/spec-index.json` 开始，只读取主能力 Spec 的 Implementation Map；扩大搜索后必须补回缺失映射。
3. 用户可观察行为、契约、持久化格式或安全边界变化，先创建 `openspec/changes/<change-id>/`，实现完成后同步当前能力 Spec。
4. Rust 依赖方向固定为 `interface -> application -> domain`、`infrastructure -> application ports + domain`；domain 不依赖 Tauri/reqwest，application 不依赖 infrastructure/interface。
5. `src-tauri/src/lib.rs` 只负责 adapter 装配、依赖注入、Tauri 组合和 IPC 注册；业务逻辑进入 domain/application。
6. React 使用 `app / features / shared`：shared 不依赖 feature/app，feature 不依赖 app，也不跨 feature 直接耦合。
7. 订阅 URL、UUID、密码和 token 都是秘密；日志、错误详情、测试夹具和生成报告不得记录真实值。
8. canonical URL 与 GitHub 镜像产生的 effective URL 分离；镜像变化不得改变来源身份。
9. 规则冲突不得作为重复项静默删除；唯一 `MATCH` 必须在最后。
10. 来源识别、草案就绪、配置生成、Mihomo 验证、文件导出、Clash Verge 加载、规则命中和最终出口分别报告。
11. 变更后至少运行 `npm run check:spec` 和相关能力测试；路径、IPC 或 contract 变化必须同步 `spec-index.json`。
12. 未收到明确指令时，不提交、不推送、不打 tag、不发布。
