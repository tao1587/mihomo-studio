# Proposal: 支持 WireGuard 节点文本转换

状态：`archived`

接受依据：用户于 2026-08-24 明确要求独立“节点转换”支持标准 WireGuard `[Interface]` / `[Peer]` 客户端文本，并要求转换结果不要隐藏。

## 现状证据与问题

- 独立节点转换入口、前后端 DTO 和 IPC 目前均以 VLESS 命名，只接受单条、多行或 Base64 VLESS URI；
- WireGuard 多行 INI 文本会进入 VLESS 专用转换器并以“不支持的节点来源”失败；
- 转换结果虽然没有脱敏或删除凭据，但位于默认关闭的 `<details>` 中，用户转换后仍需再次展开；
- 当前 Spec 同样把独立转换结果规定为默认折叠，因此不能只改界面而不更新安全呈现契约。

## 用户可观察行为

1. “节点转换”自动识别既有 VLESS 输入和一份标准 WireGuard 客户端配置；
2. WireGuard 输入包含一个 `[Interface]` 和一个 `[Peer]`，转换为一个 Mihomo `type: wireguard` 节点；
3. 映射 `PrivateKey`、`Address`、`DNS`、`PublicKey`、`Endpoint`、`AllowedIPs` 与 `PersistentKeepalive`，并保留完整私钥和公钥到敏感 YAML；
4. 没有名称的 WireGuard 配置使用稳定的非秘密名称 `WireGuard 1`；
5. 用户主动转换成功后，完整 `proxies:` YAML 在独立转换页直接可见并可显式复制，不再默认折叠；
6. VLESS 的单条、多行、Base64、去重、重名与兼容归一化行为保持不变。

## 影响能力与实现

- `source-ingestion`：增加 WireGuard INI 的独立片段转换，转换契约和 IPC 改为协议中性命名；
- `desktop-experience`：输入文案覆盖 VLESS/WireGuard，独立转换结果默认直接显示；
- `security-privacy`：只放宽用户主动转换后的当前页面披露，不改变私钥和 YAML 的秘密分类；
- `system-architecture`：IPC 从 `convert_vless_node_text` 重命名为 `convert_node_text`，同步 Rust、TypeScript 与 `spec-index.json`；
- 主要实现位于 `src-tauri/src/domain/compile/node.rs`、source application/interface adapter、`src/features/node-converter/` 和 shared contract/API。

## 安全、隐私、兼容与回滚

- WireGuard 私钥、完整输入和生成 YAML 仍是秘密，只存在当前 React 状态、当前 IPC 参数/结果、Rust 调用内存和用户主动复制的剪贴板；
- 不写日志、错误详情、生成报告、缓存、文件或持久化；错误只返回稳定类别，不回显字段值、Endpoint 或密钥；
- 常显 YAML 不放入 live-region，避免辅助技术把私钥作为状态更新自动播报；
- 测试只使用保留地址、`.invalid` 域名和明显占位密钥，不复制用户输入；
- 不保留旧 VLESS IPC 名称；前端与后端在同一变更中原子迁移。回滚时可恢复旧 IPC/DTO、移除 WireGuard parser，并恢复默认折叠结果。

## 非目标

- 不把 WireGuard INI 接入完整配置编译、来源检查或订阅识别；
- 不支持多 `[Peer]`、wg-quick 的路由/脚本字段、文件导入或自动导入 Clash Verge；
- 不自动写剪贴板，不验证节点连接、规则命中或最终出口；
- 不改变完整严格配置 YAML、订阅 URL 或其他秘密界面的默认折叠/遮罩策略。

## 验收场景

### 单 Peer WireGuard 转换

- **GIVEN** 一份使用占位密钥、保留地址和一个 Peer 的 WireGuard 客户端文本
- **WHEN** 调用协议中性的节点转换 IPC
- **THEN** 输出一个 `proxies:` 下的 Mihomo WireGuard 节点
- **AND** IPv4 Address、DNS、Endpoint host/port、AllowedIPs 与 keepalive 映射到目标字段
- **AND** 私钥、公钥保留在敏感 YAML 中，不进入无秘密状态摘要。

### 不兼容 WireGuard 输入

- **GIVEN** 配置缺少必要字段、字段重复、CIDR/端点/端口无效、包含多个 Peer 或包含未映射字段
- **WHEN** 用户转换
- **THEN** 转换全量失败且不返回部分 YAML
- **AND** 错误不回显密钥、Endpoint、Address、DNS 或其他字段值。

### 结果直接呈现

- **WHEN** 用户主动转换成功
- **THEN** 敏感 YAML 无需再次展开即可见并可显式复制
- **AND** 修改输入立即清除旧结果
- **AND** 页面仍说明未导入、未连接、未验证出口。

## 证据边界

- Rust 单元测试证明输入识别、字段映射、fail-closed 和秘密不回显；
- 前端测试与构建证明协议中性契约、输入提示和直接呈现门禁；
- `npm run check:spec` 证明 IPC/索引/能力文档同步；
- 固定 Mihomo sidecar 对纯占位片段或嵌入 fixture 的接受性验证只证明目标内核接受语法；
- 本变更不证明文件已导出、Clash Verge 已加载、节点已连接、规则已命中或最终公网出口可用。

## 完成证据

- `npm run check:spec` 通过：8 个能力、6 个 IPC 命令，架构检查覆盖 36 个 Rust 文件和 23 个前端文件；
- 前端 6 个测试文件、20 个测试通过，TypeScript/Vite 生产构建通过；
- Rust 62 个测试通过，1 个既有 loopback socket 权限测试忽略；`cargo fmt --check` 与 `cargo check` 通过；
- 固定 SHA-256 的官方 Mihomo v1.19.29 实际接受原有 3 份配置与新增的纯占位 WireGuard 节点 fixture；
- 独立审查发现并修复在途旧结果回流与 Address CIDR 前缀丢失，复审后无剩余 P0–P3 问题；
- 未执行文件导出、Clash Verge 加载、节点连接、规则命中或最终出口验证；当前目录没有 Git 元数据，未初始化仓库、提交或推送。
