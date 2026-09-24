# Proposal: 在节点输入区增加 VLESS 转换预览

状态：`archived`

后续更正：入口布局已被 [`separate-vless-converter-entry`](../separate-vless-converter-entry/proposal.md) 取代；本文件只保留当时的历史决策。

## 问题

当前严格编译路径已经能把 VLESS 分享链接转换为 Mihomo Node IR，但用户只有在生成完整配置时才能间接获得转换结果。只想把一条或多条 VLESS 分享链接转换成 Clash/Mihomo `proxies` 节点片段时，界面没有独立入口，也不能直接检查字段映射。

## 目标

1. 在“直接粘贴节点”卡片内增加次级“转换预览”操作，复用现有多行输入，不新建重复输入区或顶级步骤；
2. 复用严格编译器的权威 VLESS 分享链接转换器，支持单条、多行及 Base64 VLESS URI 文本；
3. 输出可直接合并进 Clash/Mihomo 配置的 `proxies:` YAML 片段；
4. 结果默认折叠并明确标记为敏感内容，可一键复制；
5. 多节点名称冲突自动分配稳定后缀，完全重复节点精确去重；
6. 目标内核无等价字段只按既有显式兼容规则归一化，并返回无秘密数量提示；其他未知或不兼容参数继续 fail-closed。

## 非目标

- 不生成策略组、规则、DNS/TUN 或完整配置；
- 不自动把转换结果写入文件、剪贴板、日志或持久化存储；
- 不把转换成功表述成 Mihomo 已加载、Clash Verge 已导入或节点出口可用；
- 不扩展 VMess、Trojan、SS 等其他分享协议。

## 交互决策

转换属于“直接节点输入”的即时派生操作，而不是新的配置阶段。因此入口与“识别并添加”并列放在同一编辑器页脚：主操作继续负责把来源加入工作区，次级操作只生成节点片段预览。转换结果紧邻输入出现，默认折叠，避免在页面中制造第四个顶级步骤或第二个秘密输入框。

## 安全、兼容与回滚

- 分享链接和转换 YAML 只存在于当前 React 状态、当前 IPC 参数/结果和 Rust 调用内存；
- 错误继续使用已有的来源/节点序号与稳定类别，不回显 URI、名称、server、UUID、参数名或参数值；
- 回滚副本：`/tmp/mihomo-studio-before-inline-vless-converter-20260819`；
- 当前目录没有 Git 元数据；不初始化仓库，不提交或推送。

## 验收场景

### 单条 REALITY VLESS

- **GIVEN** 一条包含 `security=reality`、TCP、SNI、fingerprint、public key 和 short id 的 VLESS 分享链接
- **WHEN** 用户点击“转换预览”
- **THEN** 返回一个 `proxies:` YAML 节点，使用 Mihomo 的 `servername`、`client-fingerprint`、`reality-opts` 和 `network` 字段
- **AND** 输入与输出原文不进入提示或错误。

### 多条链接

- **GIVEN** 多条换行分隔的 VLESS 分享链接
- **WHEN** 用户转换
- **THEN** 输出同一个 `proxies:` 列表
- **AND** 重名节点获得稳定后缀，完全重复节点只保留一个并报告去重数量。

### 不兼容字段

- **GIVEN** 链接含未知、重复或与 security/transport 不兼容的参数
- **WHEN** 用户转换
- **THEN** 转换失败并显示脱敏问题类别
- **AND** 不生成部分 YAML。

## 证据边界

源码、单元测试与前端构建只证明转换契约和 UI 门禁；不证明文件导出、Mihomo/Clash Verge 加载、规则命中或最终公网出口。

## 完成证据

- Rust 全量测试 59 passed、1 个 loopback socket 环境测试 ignored；
- `cargo fmt --check`、`cargo check`、Clippy `-D warnings` 通过；
- 前端 13 个测试、TypeScript/Vite 生产构建与 `npm run check:spec` 通过；
- 固定 Mihomo v1.19.29 接受 3 个现有 golden configuration；
- 浏览器预览在占位 VLESS 输入下显示 1 行、两个操作均可用、运行时边界错误就地呈现且控制台无 warning/error；
- 未执行文件导出、应用安装/替换、Clash Verge 加载、节点连接或最终出口验证。
