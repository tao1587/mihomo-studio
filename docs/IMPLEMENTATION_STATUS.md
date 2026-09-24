# 实施状态

更新时间：2026-08-14

## 已完成：Phase 1 基础骨架

- 官方 Tauri 2 + React + TypeScript 工程；
- 项目目录切换到 `/Users/ming/Sites/mihomo-studio`；
- Rust catalog/profile 模块与 Tauri IPC；
- 简单模式和完全模式 UI；
- 订阅、直接节点的内存输入和显示遮罩；
- GitHub 镜像输入；
- 10 个仓库级规则来源；
- 12 个首批具体 rule-set 条目；
- 默认 ACL4SSR 文本 family；
- MetaCubeX opaque MRS 边界；
- routing family 和广告 family 互斥元数据；
- 唯一 `MATCH,其他兜底` 结构预览；
- 前端与 Rust 测试；
- Tauri debug no-bundle 构建。

## 已完成：Phase 2 第一批节点来源能力

- Rust `SourceService` 领域模块和两个 Tauri 命令：
  - `inspect_subscription`；
  - `inspect_node_text`；
- 订阅抓取支持系统代理和显式直连两条路径；
- 依次探测 Mihomo、Clash.Meta、Clash Verge、浏览器 User-Agent；
- 20 秒总超时、5 次重定向、8 MiB 响应体上限；
- 阻止远程 HTTPS 降级到 HTTP、远程重定向到 loopback、URL authority 凭据；
- 识别 URI 列表、Base64 URI、Mihomo `proxies` YAML 和 provider YAML；
- VLESS、VMess、Trojan、SS/SSR、Hysteria2、TUIC、WireGuard 等协议归类；
- 节点显示名称不参与指纹，URI 查询参数排序后计算 SHA-256；
- 返回 UI 的来源摘要不包含订阅 path、query、fragment、节点凭据或显示名称；
- UI 显示来源格式、识别节点数、协议分布、去重数和 resolver 状态；
- 未解析出节点的来源不会被标记为“可进入编译”；
- 本机回环集成测试验证前三个 User-Agent 自动回退并识别 Base64 订阅。

## 当前没有宣称实现

- Mihomo sidecar 解析；
- last-known-good 订阅缓存；
- 操作系统凭据库存储；
- 规则文件下载与缓存；
- 内容级规则去重；
- Mihomo YAML 生成；
- Clash Verge 导入或运行态验证。

UI 对这些阶段只展示结构草案，不显示“配置已生成”。

## 下一阶段

1. 短生命周期 Mihomo resolver；
2. secret reference 与操作系统凭据库；
3. last-known-good 订阅缓存；
4. RuleSource 下载、cache/hash 与镜像重写；
5. Rule IR 和去重报告。
