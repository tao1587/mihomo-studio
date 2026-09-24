# Capability: 安全与隐私

| 字段 | 值 |
|---|---|
| ID | `security-privacy` |
| 状态 | `partial` |
| 数据原则 | 本地优先、最小持久化、默认脱敏 |

## Purpose

定义秘密、远程输入、日志、缓存、报告、sidecar 与导出文件的安全边界，避免在功能扩展时把用户订阅和节点凭据变成普通配置数据。

## Data Classification

### Secrets

- 订阅完整 URL；
- URL query、token、UUID、密码和私钥；
- 节点完整 URI/YAML；
- sidecar Controller secret；
- 导出 YAML 中的节点凭据。

### Non-secret identifiers

- 内部 source ID；
- 不含 path/query/fragment 的脱敏 host 标签；
- 内容格式和协议计数；
- 无法反推秘密的内容 hash；
- catalog 中公开的 canonical repository metadata。

## Current State

- 来源秘密目前只存在 React state、当前 IPC 参数和抓取/解析内存；
- Rust 抓取器限制 URL、重定向、超时和响应体；
- 返回 UI 的摘要省略 URL 路径与凭据；
- Profile 草案包含严格 DNS 与出口隐私编译契约，并明确禁止 `DIRECT` 用户流量；
- strict compiler 只在当前 React/IPC/内存调用中处理 source material 与 bootstrap mapping；VLESS URI 的 UUID、Reality key、SNI、path/host、节点名、节点域名/IPv4 映射以及 secret-bearing 请求、Node IR 和 YAML 结果不实现调试输出；REALITY `spx`/`pqv` 目标兼容警告只报告字段类别和受影响节点数，不报告值；
- 独立节点转换器只在当前 React/IPC/内存调用中处理 VLESS 或 WireGuard 原文、`proxies:` YAML 及可选的完整内置模版 YAML；用户主动转换成功后结果在该独立页面直接可见，只有用户主动操作才写入剪贴板，DTO 不实现调试输出；内置模版资源已移除附件原节点和名称；
- 完整配置 YAML 仍默认折叠展示，生成报告只含计数、状态和内容 hash；
- 尚无持久化日志、来源存储、凭据库、缓存、绑定 sidecar 和导出实现。

## Requirements

### Requirement: Secret lifecycle

秘密 SHALL 只在完成当前操作所需的最小范围内存在。持久化来源前必须把 secret value 存入操作系统凭据库，普通配置只保存不可逆用途明确的 secret reference。

#### Scenario: 保存订阅来源

- **WHEN** 未来用户选择持久化订阅
- **THEN** metadata 与 secret value 分离保存
- **AND** metadata 文件不包含 URL query、token 或节点凭据

#### Scenario: 预览转换后的节点片段或完整模版

- **WHEN** 用户把 VLESS 分享链接或 WireGuard 客户端文本转换为 `proxies:` YAML
- **THEN** 输入和节点片段、可选完整模版只存在于当前 React 状态、IPC 参数/结果和 Rust 调用内存
- **AND** 用户主动转换成功后 YAML 在独立转换页直接可见，但不进入 live-region
- **AND** 不自动写文件、日志、缓存或剪贴板
- **AND** 只有用户主动点击复制时才进入剪贴板。

### Requirement: 日志与错误

日志、错误详情、遥测、测试报告和生成报告不得包含 secrets。错误 SHALL 使用稳定类别、安全来源 ID 和必要的脱敏 host。

#### Scenario: 网络错误

- **WHEN** 订阅请求连接失败
- **THEN** 错误可描述“连接失败”和安全来源标签
- **AND** 不包含完整请求 URL、响应体或凭据

### Requirement: 远程请求边界

远程订阅和规则抓取 SHALL：

- 默认要求 HTTPS；
- 拒绝 authority credentials；
- 限制重定向、时间和大小；
- 阻止远程到 loopback/private 的危险跳转；
- 把 proxy/mirror transport 与 canonical identity 分离；
- 不执行远程脚本。

规则下载器接入前必须复用或强化来源抓取的安全策略，不得另写宽松 HTTP 路径。

### Requirement: 缓存

last-known-good 缓存 SHALL 以内容寻址并记录来源 ID、内容 hash、时间和验证状态。缓存中若包含节点凭据，必须使用受限权限；失效刷新不得覆盖最后一个已验证快照。

### Requirement: Sidecar

Mihomo sidecar SHALL：

- 只监听 loopback；
- 使用随机临时 Controller secret；
- 使用独立临时 HomeDir；
- 禁用主动健康检查；
- 限时运行并在所有退出路径清理；
- 不将 secret 写入命令日志或错误报告。

### Requirement: 导出文件

导出 YAML 被视为 secret-bearing artifact。系统 SHALL 使用私有权限、避免写入世界可读临时目录，并在 UI 中提示用户不要公开分享。无秘密生成报告必须与 YAML 分开保存。

### Requirement: DNS 与原始出口隐私

严格隐私编译 SHALL：

- 让应用 DNS 进入 Mihomo 内部 DNS，并让上游加密 DNS 连接使用指定代理策略；
- 拒绝系统 DNS、普通 53 或直连 DoH fallback；
- 拒绝无外层代理、已验证本地映射或其他受保护路径的节点域名 bootstrap；
- 用户确认的节点 IPv4 映射只允许按来源/节点位置消费并写入敏感 YAML `hosts`，不得进入报告或错误；
- 拒绝 remote provider 的直连下载或刷新；
- 要求 TUN DNS hijack、auto route 与 `strict-route`；
- 在 IPv6 DNS、路由和节点出口闭环验证前关闭 IPv6；
- 禁止用户目标流量落到 `DIRECT`，包括国内规则。

#### Scenario: 加密但直连的 DoH

- **GIVEN** DoH 使用 HTTPS
- **AND** DoH 连接通过 `DIRECT`
- **WHEN** 严格隐私检查运行
- **THEN** 检查失败
- **AND** 不把“DNS 已加密”解释为“原始出口未暴露”。

#### Scenario: 用户提供已验证 bootstrap 映射

- **GIVEN** 域名节点拥有通过受保护路径确认的 IPv4
- **WHEN** 用户按来源/节点位置提交映射
- **THEN** 编译器把域名到 IPv4 写入当前 YAML 的 `hosts`
- **AND** 不调用系统 DNS 或直连 DoH 补全映射
- **AND** 报告、错误和测试输出不包含域名或 IPv4。

### Requirement: 隐私证据分层

草案只证明隐私意图，静态 YAML 只证明配置不变量，Mihomo 加载只证明内核接受配置。Clash Verge 当前配置、DNS 连接路径、规则命中、IPv4/IPv6 与应用层旁路检查 SHALL 分别提供运行态证据；较早状态不得推出较晚状态。

### Requirement: 测试夹具

测试 SHALL 使用保留域、loopback、占位 UUID/密码和人工构造 token。真实订阅 URL、账号、节点或设备标识不得进入 fixture、snapshot 或 golden file。

#### Scenario: 新增协议 fixture

- **WHEN** 为 VLESS、WireGuard 或其他协议添加测试样本
- **THEN** server 使用保留域或 loopback
- **AND** secret 字段使用明显占位值
- **AND** 测试断言输出摘要不回显这些值

### Requirement: 公共 CI 与发布边界

公共 GitHub Actions SHALL 使用最小令牌权限并固定第三方 Action 的完整 commit SHA。CI 和 Release 不得把订阅 URL、UUID、密码、token、生成 YAML 或其他 secret-bearing 文件作为日志或构建产物上传。

#### Scenario: 普通 CI 运行

- **GIVEN** 仓库包含 secret-bearing 运行时功能
- **WHEN** push 或 pull request 触发 CI
- **THEN** 工作流只使用仓库内的占位测试数据
- **AND** `GITHUB_TOKEN` 只有 `contents: read`。

## Implementation Map

| 路径 | 当前责任 |
|---|---|
| `src-tauri/src/domain/source/model.rs` | URL 值对象、typed error 与安全摘要模型 |
| `src-tauri/src/domain/source/parser.rs` | 数据解析和无名称指纹 |
| `src-tauri/src/application/source.rs` | 订阅探测用例与无秘密尝试摘要 |
| `src-tauri/src/infrastructure/source/reqwest_gateway.rs` | 重定向、超时和响应边界 |
| `src-tauri/src/interface/ipc/source.rs` | Tauri 错误映射 |
| `src-tauri/src/domain/profile/model.rs` | 严格隐私草案 contract |
| `src-tauri/src/domain/profile/service.rs` | 严格隐私草案不变量 |
| `src-tauri/src/domain/compile/` | secret-bearing Node IR、严格 serializer、回读与安全错误 |
| `src-tauri/src/domain/compile/template.rs` | 去敏内置模版与敏感结果组装，不记录节点内容 |
| `src-tauri/src/domain/compile/templates/clash-single-node.yml` | 已移除原节点秘密和名称的模版资源 |
| `src-tauri/src/application/compile.rs` | 当前调用的 secret material 生命周期 |
| `src-tauri/src/interface/ipc/compile.rs` | 编译 IPC 安全错误映射 |
| `src/features/source-inspection/` | 遮罩输入和安全摘要展示 |
| `src/features/node-converter/` | 独立 secret-bearing 转换状态、会话内直接可见结果与主动复制 |
| `src/shared/api/studio.ts` | 前端 IPC 边界 |
| `src/shared/contracts.ts` | 前端严格隐私 DTO |
| `src/features/profile-preview/ui/PreviewPanel.tsx` | 隐私约束与证据边界文案 |
| `src-tauri/tauri.conf.json` | WebView CSP |
| `.github/workflows/ci.yml` | 公共 CI 的只读权限和本地门禁 |
| `.github/workflows/release.yml` | 标签发布的最小写权限与非秘密安装包产物 |
| `scripts/check-github-workflows.mjs` | 工作流权限和固定 Action 引用检查 |

凭据库、缓存、绑定 sidecar 和导出路径尚未存在；实现时必须先更新本 Spec 和 Implementation Map。

## Verification

```bash
cd src-tauri && cargo test source::
cd src-tauri && cargo test profile::
npm test
```

发布前另需进行 fixture secret 扫描、日志审计、文件权限验证和 sidecar 残留检查。
