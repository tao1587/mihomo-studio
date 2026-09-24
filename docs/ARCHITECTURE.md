# 技术架构

## 1. 推荐技术栈

- 桌面壳：Tauri 2；
- 前端：React + TypeScript + Vite；
- 配置编译核心：Rust；
- YAML：保留顺序和可读性的安全解析/序列化库；
- 秘密存储：操作系统凭据库，应用配置只保存 secret reference；
- 缓存：应用数据目录中的内容寻址文件；
- 验证：绑定并校验 hash 的 Mihomo sidecar；
- 旧格式兼容：可替换的 subconverter 适配器，不让它成为核心数据模型。

选择 Rust 核心的原因：跨平台行为一致、适合受限网络请求和进程管理、便于实现规则 IR、CIDR 树和确定性输出。

## 2. 模块划分

```text
UI / Wizard
    |
Tauri Commands
    |
Application Services
    |-- SourceService
    |-- RuleCatalogService
    |-- GroupDesignerService
    |-- CompileService
    |-- ValidationService
    |-- ExportService
    |
Domain Core
    |-- Node IR
    |-- Rule IR
    |-- Policy/Group Graph
    |-- Dedup + Conflict Engine
    |-- Mihomo Compiler
    |
Adapters
    |-- HTTP/GitHub Fetcher
    |-- URI/Base64/YAML Parsers
    |-- Ephemeral Mihomo Resolver
    |-- Optional Subconverter Adapter
    |-- Keychain
    |-- Filesystem/Cache
```

业务逻辑不放在 `lib.rs`；Tauri 入口只负责组合、命令注册和生命周期。

## 3. 节点来源解析管线

### 3.1 获取

1. Rust 后端发起请求，绕开 WebView CORS；
2. 限制重定向次数、响应体大小和超时；
3. 根据来源配置依次尝试 User-Agent：
   - 已记录成功的 UA；
   - `mihomo`；
   - `Clash.Meta`；
   - `Clash-Verge`；
   - 常规浏览器；
4. 记录响应 hash、ETag、Last-Modified，不记录秘密 URL；
5. 失败时使用 last-known-good 快照，并清楚标记“缓存数据”。

### 3.2 内容识别

识别依据是响应内容而不是扩展名：

1. Mihomo YAML（`proxies`）；
2. proxy-provider YAML；
3. 逐行 URI；
4. Base64 后的逐行 URI；
5. 单个 URI；
6. 可选的旧 subconverter 输入；
7. 未识别则显示原因为“格式未知”，不把登录页/错误页误当订阅。

### 3.3 Mihomo 解析适配器

官方 Mihomo proxy-provider 可以接收 YAML、URI 和 Base64 内容。对本地解析器不能完整覆盖、但 Mihomo 客户端能够识别的来源：

1. 在临时 HomeDir 生成最小配置；
2. 使用随机 loopback Controller 端口和临时 secret；
3. 健康检查关闭，避免主动连接节点；
4. 由绑定版本的 Mihomo 加载 proxy-provider；
5. 从本地 Controller 读取规范化节点；
6. 立刻关闭 sidecar 并删除临时秘密文件；
7. 将解析结果转换为内部 Node IR。

这比在桌面应用中复制所有快速变化的协议解析逻辑更可靠。subconverter 只作为旧格式的后备适配器，不再通过一个常驻 HTTP 服务组织内部流程。

### 3.4 Node IR

每个节点至少保存：

- 内部 ID；
- 显示名称和规范化名称；
- 来源 ID；
- 协议类型；
- server/port；
- 连接参数的结构化表示；
- 支持 UDP 的已知状态；
- 地区识别结果和置信度；
- 内容指纹；
- secret reference。

节点去重分两层：

- 精确内容指纹相同：合并来源引用；
- 端点相同但凭据/传输参数不同：视为冲突候选，不自动合并。

名称重复不代表节点重复。重名节点使用稳定来源标签消解，如 `[SOURCE] NAME`。

## 4. GitHub 镜像模型

用户配置的 `proxy_url` 只影响下载路径，不改写规则源的 canonical URL。

```text
canonical_url
  -> 判断是否为 github.com/raw.githubusercontent.com/gist.githubusercontent.com
  -> 规范化 proxy_url 末尾斜杠
  -> effective_url = proxy_url + canonical_url
  -> 下载失败时按用户配置决定是否直连回退
```

规则源可以覆盖全局镜像。缓存和来源报告始终按 canonical URL/hash 标识，防止更换镜像后产生重复来源。

镜像服务可以看到被请求的仓库路径，UI 需要提示这一隐私边界。

## 5. 规则编译管线

```text
Rule Sources
  -> Fetch/Cache
  -> Format Parser
  -> Rule IR
  -> Policy Binding
  -> Priority Sort
  -> Exact Dedup
  -> Conservative Semantic Dedup
  -> Conflict Resolution
  -> Provider Partition
  -> Ordered Mihomo Rules
```

Rule IR 字段：

- matcher type；
- normalized value；
- flags（例如 `no-resolve`）；
- target policy；
- source ID、行号、原始文本；
- source priority 和 user priority；
- content hash；
- dedup/conflict decision。

## 6. 策略组图

策略组不是字符串拼接，而是有向图：

- 节点可以引用代理、provider 或其他组；
- 编译前做拓扑检查；
- 禁止循环；
- 空组必须有显式回退；
- 动态国家组只从本次实际节点生成；
- 规则目标必须引用存在的组或内置动作；
- 所有特殊服务规则位于宽泛地理规则和最终 `MATCH` 之前。

## 7. 配置输出与验证

输出步骤：

1. 从领域模型生成确定性 YAML；
2. YAML 解析回读；
3. 校验节点、provider、组名称唯一；
4. 校验所有引用存在；
5. 校验组图无环；
6. 校验规则结构和目标；
7. 校验唯一 `MATCH` 且位于最后；
8. 用绑定版本 Mihomo 验证；
9. 写临时文件并 fsync；
10. 原子替换目标文件，同时保留 last-known-good；
11. POSIX 下权限设为 `0600`，Windows 下限制为当前用户。

报告需要分层：

- **已生成**：编译器输出成功；
- **Mihomo 已验证**：指定内核通过配置加载；
- **已导出**：文件写入目标路径；
- **Clash Verge 已加载**：只有后续通过外部应用状态确认才可显示；
- **实际规则命中/出口已验证**：需要运行态 Controller 和实际流量，不能由静态验证推断。

## 8. 安全和隐私

- 订阅 URL、节点密码、UUID、token 都视为秘密；
- 日志只使用来源 ID、脱敏主机和错误类别；
- 应用元数据与秘密分离；
- 远程 YAML 只做数据解析，不执行脚本；
- 限制响应大小、下载时间、重定向和本地文件读取边界；
- rule-provider path 必须唯一且位于生成配置 HomeDir；
- sidecar 只监听 loopback，并使用临时 secret；
- 自动更新失败时不覆盖最后一个有效快照；
- 生成报告不包含 canonical URL 的 query/token；
- 导出的 YAML 本身含节点凭据，UI 必须明确提示妥善保存。

## 9. 许可证边界

- ACL4SSR 当前为 CC BY-SA 4.0；
- MetaCubeX/subconverter 为 GPL-3.0；
- MetaCubeX/meta-rules-dat 为 GPL-3.0；
- blackmatrix7/ios_rule_script 为 GPL-2.0；
- 历史失效规则仓库不进入候选、依赖、内置 catalog 或兼容层。

建议项目整体采用 GPL-3.0-or-later，分发第三方二进制、规则快照和派生预设时附带许可证、源码地址、版本/commit 和归属说明。若决定使用其他许可证，需把“运行时远程目录”与“打包进安装包的代码/规则数据”严格拆开并做专项许可证评审。
