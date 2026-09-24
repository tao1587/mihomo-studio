# 节点来源接入

## 当前数据流

```text
订阅 URL / 直接节点文本
  -> Tauri IPC（仅当前调用内存）
  -> URL 安全检查或本地文本大小检查
  -> 多 User-Agent HTTP 抓取
  -> 内容识别
  -> 节点指纹与单来源精确去重
  -> 无秘密的 SourceInspectionSummary
  -> UI 来源卡片
```

业务逻辑位于 `src-tauri/src/source/`。`src-tauri/src/lib.rs` 只注册命令。

## 抓取路径

- `system`：使用操作系统代理发现，适合当前已开启 Clash Verge 系统代理的环境；
- `direct`：显式关闭代理，用于诊断或本地 fixture；
- 当前 UI 默认选择 `system`，完全模式的逐来源切换后续接入；
- 远程订阅要求 HTTPS；HTTP 仅允许 `localhost`、`127.0.0.0/8` 和 `::1`；
- URL authority 中的用户名/密码被拒绝；
- 最多 5 次重定向，禁止 HTTPS 降级和远程地址跳转到 loopback；
- 请求总超时 20 秒，响应体最多 8 MiB。

User-Agent 顺序：

1. `mihomo/1.19`；
2. `Clash.Meta`；
3. `Clash-Verge`；
4. Mihomo Studio 浏览器兼容标识。

只有内容被识别后才停止探测；HTTP 错误、HTML 页面和未知格式会继续尝试下一种 User-Agent。

## 当前识别格式

- 一行或多行代理 URI；
- Standard / URL-safe、带 padding / 无 padding Base64 文本；
- Mihomo YAML 顶层 `proxies`；
- Mihomo YAML 顶层 `proxy-providers`（标记为需要 resolver）；
- UTF-8 以外的 provider/binary（标记为需要 resolver）；
- HTML 错误页（识别为错误内容，不计节点）。

本地 parser 只做数据解析，不执行远程脚本或表达式。

## 去重边界

- URI：去掉 fragment/显示名，对查询参数排序，再计算 SHA-256；
- YAML proxy：去掉顶层 `name` 字段，再计算 SHA-256；
- 相同内容但改名的节点会合并；
- 相同 endpoint 但 UUID、密码、传输或 TLS 参数不同的节点不会合并；
- 当前只报告单个来源内部的重复项；跨来源 provenance 合并属于后续 Node IR 阶段；
- 规则条目去重尚未接入，不能把节点去重结果解释为规则去重完成。

## 秘密边界

当前订阅 URL 和节点文本只保存在 React 输入状态、当前 Tauri IPC 参数和当前解析/请求内存中；不写日志、不写 catalog、不返回错误详情中的原始 URL。返回 UI 的摘要只包含：

- 脱敏 host 标签；
- 内容格式；
- 节点数和重复数；
- 协议计数；
- 成功的 User-Agent 名称；
- 不含原始内容的错误类别与警告。

目前尚未实现持久化来源。后续保存/刷新订阅时必须先接入 secret reference 和操作系统凭据库，不把 token 写入普通应用配置。

## 状态边界

- “已识别节点”表示本地 parser 得到结构和指纹；
- “需要 Mihomo resolver”表示当前 parser 不足以枚举节点；
- “可进入编译”要求至少一个来源、至少一个已识别节点和至少一个规则来源；
- 这些状态都不表示 Mihomo 已验证、YAML 已导出、Clash Verge 已加载或真实出口已验证。
