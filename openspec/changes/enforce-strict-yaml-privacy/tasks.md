# Tasks: 生成配置强制执行严格 DNS 与出口隐私

- [x] 记录目录无 Git 元数据并保存本次修改前副本；
- [x] 定义严格隐私草案 contract 与 fail-closed 边界；
- [x] 在 Rust profile domain 和 TypeScript contract 中加入隐私摘要；
- [x] 在浏览器草案 adapter 与预览 UI 中同步展示；
- [x] 为简单/完全模式增加一致性测试；
- [x] 同步当前能力 Specs、Implementation Map 和 `spec-index.json`；
- [x] 实现 Node IR 并识别 IP literal、域名 bootstrap 与受保护 bootstrap；
- [x] 接通 VLESS URI/Base64 URI 到严格 Node IR 的安全转换；
- [x] 实现 Rule IR/Provider IR，拒绝 `DIRECT` 用户流量和直连 remote provider；
- [x] 实现确定性 YAML privacy sections 与 golden/round-trip tests；
- [x] 增加固定官方版本/hash 的 darwin arm64 golden fixture 加载门禁；
- [ ] 使用绑定 Mihomo sidecar 验证 YAML；
- [ ] 实现私有权限原子导出；
- [ ] 增加 Clash Verge 加载、DNS 路径、规则命中和最终出口的独立 E2E 证据；
- [x] 运行 `npm run check:spec`、前端测试/构建和 Rust profile 测试。
