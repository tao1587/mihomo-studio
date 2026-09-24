# 产品与架构决策

## 已确认

### D1. 项目名称与目录

- 产品名：`Mihomo Studio`
- 项目目录：`/Users/ming/Sites/mihomo-studio`
- 定位：生成 Mihomo YAML，不作为代理客户端。

### D2. 规则源依赖边界

已失效的历史规则仓库不进入候选、代码依赖、内置 catalog、兼容层或安装包。NAS 中的历史副本只作为旧实现审计背景，不迁移到新项目。

### D3. 技术栈

- Tauri 2；
- React + TypeScript + Vite；
- Rust 领域核心；
- 短生命周期 Mihomo sidecar 用于原生 provider 解析和最终配置验证；
- subconverter 只保留为未来可能的旧格式适配器，不作为常驻服务或核心数据模型。

### D4. 规则源组合

- 简单模式默认：ACL4SSR 可审计文本规则 + Mihomo Studio 内置 Claude 包；
- MetaCubeX：geodata/MRS 可选源；
- blackmatrix7：完全模式按单服务选择；
- Loyalsoldier、SukkaW、DustinWin：互斥的替代 family，不与默认宽泛 family 叠加；
- v2fly/domain-list-community：登记为上游数据，不作为普通 provider 勾选。

### D5. 广告规则互斥

`ads.primary` 同时只能选择一个：

- 保守默认：ACL4SSR BanAD；
- 平衡：AWAvenue Only.Ads；
- 强力：anti-AD。

anti-AD 已聚合其他广告上游，不与它们重复启用。

### D6. 去重声明

- text/yaml：可以做内容级规范化、来源追踪和去重；
- MRS：标记为 `opaque`，只做 source/provider/hash 去重；
- 同 matcher 不同策略属于冲突，不能作为重复项静默删除；
- `DOMAIN-KEYWORD` 覆盖默认只提示；
- 唯一 `MATCH` 必须最后。

### D7. GitHub 镜像

- catalog 永久保存 repository/ref/path/canonical URL；
- `proxy_url` 只生成 effective transport URL；
- 只重写 GitHub Raw、Release 和 GitHub raw path；
- 非 GitHub 资产地址不拼接 GitHub 镜像；
- 镜像失败可回退直接源，但必须核对格式和内容 hash。

### D8. Claude 规则

- 第一方 Claude 路由默认开启；
- 本地受控规则优先于外部 Claude 聚合规则；
- 共享 GitHub、Cloudflare、Google 等基础设施不生成宽泛 Claude 域名规则；
- UDP fail-closed 和精确第三方遥测拦截默认关闭，完全模式独立开启。

### D9. 首版 Clash Verge 集成

- 生成完整 YAML；
- 用户在 Clash Verge Rev 导入一次；
- 不直接修改 Clash Verge 内部数据库或配置目录；
- loopback 托管订阅和自动刷新放到第二阶段。

## 暂定、可后续调整

### D10. 许可证

项目代码暂定 `GPL-3.0-or-later`。第三方规则和 sidecar 继续分别保留原许可证、归属、repository、ref/commit 和内容 hash。

### D11. 节点输出

计划支持 `自动 / Provider / 快照`：

- Provider 可自动更新，但导出 YAML 会保存订阅 URL；
- 快照更便于审计，但更新需要重新生成；
- 自动模式先做 Mihomo 兼容验证，不适配时退回快照。
