# Tasks: 支持 WireGuard 节点文本转换

- [x] 记录 Git 元数据缺失，不初始化仓库且不复用用户秘密作为 fixture；
- [x] 建立 source/desktop/security/system 的 delta Spec；
- [x] 将 VLESS 专用转换 DTO、application use case、IPC 和前端 adapter 改为协议中性命名；
- [x] 在领域转换器中增加单 Interface/单 Peer WireGuard INI 识别、严格校验与 Mihomo 字段映射；
- [x] 保留 VLESS 转换、去重、重名、Base64 和安全错误行为；
- [x] 更新独立转换页文案，并让成功 YAML 常显且不进入 live-region；
- [x] 增加 WireGuard 映射、失败脱敏、契约和界面呈现测试；
- [x] 同步当前能力 Specs、`project.md` 与 `spec-index.json`；
- [x] 运行 `npm run check:spec`、相关前后端测试、构建、Rust fmt/check 和占位 fixture 的 Mihomo 验证；
- [x] 完成后将本 change 标记为 `archived`。
