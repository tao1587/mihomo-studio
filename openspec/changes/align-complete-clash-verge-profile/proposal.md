# Proposal: 对齐 Clash Verge 接管后的完整 YAML

状态：`accepted`

## 问题

Mihomo Studio 的目标产物是可导入 Clash/Mihomo 的**完整 profile YAML**，不是仅含节点与规则的片段。目标用户通常已在 Clash Verge 中开启 TUN、系统代理、IPv6 和 DNS 覆写；生成器应基于这套运行环境写出自洽的完整配置，同时承认 Clash Verge 会在运行时接管同名字段。

当前实现有三处冲突：

1. 顶层和 DNS 固定 `ipv6: false`，并拒绝 IPv6 节点，与“IPv6 已开启并由 Clash Verge 接管”的目标运行环境相反；
2. `proxy-server-nameserver` 被绑定到 `节点选择`，造成解析代理节点域名时依赖尚未建立的代理组；
3. 为规避上述循环，域名节点被强制要求手工 IPv4 bootstrap，普通订阅无法按 Mihomo 原生节点域名解析路径工作。

界面还把简单/完全模式描述为会改变 DNS/TUN，但用户选择的实际只是规则复杂度。

## 目标

1. 继续生成含全局项、DNS、TUN、节点、策略组、provider 和规则的完整 Clash/Mihomo YAML；
2. 完整 YAML 使用与 Clash Verge 常用接管开关一致的双栈基线：顶层与 DNS IPv6 开启，Fake-IP 同时包含 IPv4/IPv6 地址池；
3. TUN YAML 包含 mixed stack、auto-route、auto-detect-interface、UDP/TCP 53 劫持、strict-route 与 IPv6 TUN 地址；
4. 业务 DNS 继续绑定稳定的 `节点选择` 组，代理节点域名则使用独立、可直连的 IP-literal 加密 DNS，消除 bootstrap 环；
5. 正常接受域名和 IPv6 节点 `server`，保留 Mihomo 原生解析；手工 IPv4 bootstrap 仅作为可选 `hosts` 覆盖；
6. 简单/完全模式只改变规则来源、规则粒度和策略组，不改变 DNS、TUN、IPv6 或节点解析基线；
7. 明确 Clash Verge 的 GUI 是运行态权威层：系统代理没有 profile YAML 字段，DNS 覆写会接管最终 DNS，生成成功不证明 GUI 设置或运行态已经应用。

## 完整 YAML 基线

- 全局：本机 mixed port、rule mode、warning log、进程匹配、并发连接、`ipv6: true`；
- DNS：`enable`、`ipv6: true`、`fake-ip`、IPv4/IPv6 Fake-IP 地址池、`respect-rules`、禁止 system hosts；
- 节点 DNS：`proxy-server-nameserver` 使用不带代理组后缀的 IP-literal DoH；
- 业务 DNS：`nameserver` 使用带 `#节点选择` 的 IP-literal DoH；
- TUN：`enable`、mixed、auto-route、auto-detect-interface、strict-route、UDP/TCP 53 hijack、IPv6 TUN 地址；
- 内容：`proxies`、`proxy-groups`、`rule-providers`、唯一且最后的 `MATCH`。

## Clash Verge 接管语义

- TUN 开关最终决定 `tun.enable`，GUI 中的 TUN 子字段覆盖 profile 同名字段；
- 顶层 IPv6 开关最终决定运行态 `ipv6`；
- DNS 覆写开启后，Clash Verge 的 `dns_config.yaml` 整段接管运行态 `dns`；完整 profile 中仍保留同一 DNS 基线，供完整性、自运行和其他 Mihomo 客户端使用；
- 系统代理只修改操作系统代理入口并指向 Clash mixed port，不对应 profile 内的布尔字段。

## 非目标

- 不操作 Clash Verge 开关、不写其私有配置目录；
- 不自动导入、激活或覆盖用户当前 profile；
- 不把 YAML 静态校验描述为 DNS、IPv6、规则命中或公网出口验证；
- 不改变简单/完全模式当前的规则 catalog 选择逻辑。

## 安全与回滚

- 节点、订阅和可选 hosts 映射继续按秘密处理；
- DNS 与 TUN 基线只包含公共解析器、保留地址和稳定策略组名；
- 回滚副本：`/tmp/mihomo-studio-before-clash-verge-profile-20260820T190829.tar.gz`；
- 当前目录没有 Git 元数据，不初始化仓库、不提交或推送。

## 验收场景

### 域名节点无需手工 bootstrap

- **GIVEN** 订阅节点的 `server` 是合法域名
- **WHEN** 不提供 bootstrap mapping 生成完整 YAML
- **THEN** YAML 保留该域名
- **AND** `proxy-server-nameserver` 不依赖任何代理组
- **AND** 静态校验通过。

### 双栈完整配置

- **WHEN** 任一规则模式生成完整 YAML
- **THEN** 顶层 `ipv6` 与 `dns.ipv6` 为 true
- **AND** DNS 包含 `fake-ip-range6`
- **AND** TUN 包含 IPv6 地址、strict-route 和 UDP/TCP 53 劫持。

### 可选 hosts 覆盖

- **GIVEN** 用户为域名节点显式提供 IPv4 mapping
- **WHEN** 生成 YAML
- **THEN** mapping 写入顶层 `hosts`
- **AND** mapping 仍不进入报告或错误。

### 规则模式边界

- **WHEN** 用户切换简单规则和完整规则
- **THEN** DNS、TUN、IPv6 与节点解析基线不变
- **AND** 仅规则来源、服务分组和规则粒度变化。

## 证据边界

源码、测试、golden YAML 与固定 Mihomo 校验只证明完整配置可生成并被目标内核接受；Clash Verge 的 GUI 开关、DNS 覆写内容、实际 IPv6 路径、规则命中和最终出口仍需运行态分别验证。
