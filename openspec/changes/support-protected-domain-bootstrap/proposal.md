# Proposal: 为域名节点接入用户确认的受保护 bootstrap 映射

状态：`accepted`

## 现状与问题

严格编译当前只接受 IPv4 literal `server`。当来源中的节点使用域名时，编译器会在 Node IR 阶段 fail-closed，避免用系统 DNS 或直连 DoH 解析首个代理节点。这个边界正确，但 UI 只返回“要求 IPv4 literal 或受保护 bootstrap”，没有提供把已验证 IPv4 映射交给编译器的入口，导致常见域名节点无法完成生成。

Mihomo 的 `proxy-server-nameserver` 专门负责代理节点域名解析；若它自身通过尚未建立的节点策略访问，就会形成 bootstrap 环。当前能力不应通过取消严格校验或直连 DNS 来绕过这个环。

## 目标

1. 允许用户按“来源序号 + 节点序号”提供通过受保护路径确认的 IPv4 映射；
2. 编译器只把映射写入 secret-bearing YAML 的顶层 `hosts`，不写入生成报告、警告或错误；
3. 域名节点缺少映射时继续 fail-closed，不静默调用系统 DNS或直连 DoH；
4. 拒绝无效、重复、悬空或用于非域名节点的映射；
5. UI 给出可执行且不回显节点域名的输入格式和错误提示。

## 非目标

- 不在应用内自动查询节点域名；
- 不把 DNS 加密等同于 bootstrap 已隐藏原始出口；
- 不声明 Clash Verge 已加载、DNS 路径或最终出口已验证；
- 不持久化节点域名、IPv4 映射或来源秘密。

## 契约

`CompileProfileRequest` 新增 `bootstrapMappings[]`：

- `source`：从 1 开始的来源序号；
- `node`：从 1 开始的节点序号；
- `ipv4`：用户通过受保护路径确认的 IPv4 literal。

前端文本入口每行使用 `source:node=IPv4`。该文本只在当前 React state 中存在；调用 IPC 前转换为 typed DTO。

## 安全、兼容与回滚

- `bootstrapMappings` 与来源内容一样按 secret-bearing 请求处理，不进入报告和错误；
- 旧调用方省略字段时由 Rust serde 默认为空，IPv4 节点行为与现有 golden YAML 保持不变；
- 回滚时删除新字段和 UI 入口，域名节点恢复为无条件 fail-closed；
- 当前目录没有 Git 元数据，不初始化仓库。本次修改前状态以现有工作区文件为准。

## 验收场景

### 场景：域名节点带已验证映射

- **GIVEN** 第一个来源的第一个节点使用域名 server
- **AND** 请求包含 `source=1,node=1,ipv4=192.0.2.10`
- **WHEN** 严格编译运行
- **THEN** 节点保留原始域名
- **AND** YAML 顶层 `hosts` 包含该域名到 IPv4 的静态映射
- **AND** 静态回读确认每个域名节点都有 IPv4 hosts 映射。

### 场景：缺少映射

- **WHEN** 域名节点没有对应 bootstrap 映射
- **THEN** 编译失败
- **AND** 错误指出来源/节点序号与输入格式
- **AND** 不回显域名、节点名或凭据。

### 场景：悬空映射

- **WHEN** 映射指向不存在的节点或 IPv4 server 节点
- **THEN** 编译失败
- **AND** 不把该映射静默写入 YAML。

