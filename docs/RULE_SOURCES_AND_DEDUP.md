# 规则源目录与去重设计

完整 GitHub 生态调研、热度/维护快照和官方证据见 [RULE_ECOSYSTEM_RESEARCH.md](./RULE_ECOSYSTEM_RESEARCH.md)。

## 1. 内置规则源分层

### 默认 family

**ACL4SSR** 作为简单模式默认：

- 文本规则可审计；
- 与现有 NAS 使用习惯接近；
- BanAD 相对保守；
- 可为后续 MRS 输出保留同源审计输入。

默认启用的规则项：

1. `acl4ssr.local-area-network`
2. `acl4ssr.ban-ad`
3. `builtin.claude.route`
4. `acl4ssr.download`
5. `acl4ssr.proxy-lite`
6. `acl4ssr.china-domain`
7. `acl4ssr.china-company-ip`
8. `builtin.geoip-cn`
9. `builtin.match`

### Mihomo 原生资源

**MetaCubeX/meta-rules-dat** 作为 geodata/MRS 可选源：

- MRS 加载效率高；
- 与 Mihomo 能力同步；
- MRS 内容本身不透明，只做 provider 级去重；
- 不与 Loyalsoldier、DustinWin 的宽泛集合同时默认启用。

### 完全模式单服务目录

**blackmatrix7/ios_rule_script** 只按服务启用：

- AI、流媒体、社交、游戏等分类丰富；
- 本身是大型聚合层；
- 不提供“整个仓库全部启用”按钮；
- 每个服务目录单独核对上游、许可证和重叠。

### 完整替代 family

以下 family 与默认宽泛 family 互斥：

- Loyalsoldier/clash-rules：可审计文本核心 family；
- SukkaW/Surge：专家 family，固定 `domainset -> non_ip -> ip`；
- DustinWin/ruleset_geodata：聚合完成的 Mihomo 整套预设。

### 广告专项

`ads.primary` 三选一：

| 级别 | Source ID | 说明 |
|---|---|---|
| 保守默认 | `acl4ssr.ban-ad` | 体量较小，误杀风险相对低 |
| 平衡 | `awavenue.ads-only` | 只包含广告，不额外混入隐私类别 |
| 强力 | `antiad.standard` | 聚合范围大，需要白名单和回滚 |

## 2. Catalog 身份

规则来源不能只用最终 URL 标识。稳定身份包含：

```yaml
id: acl4ssr.ban-ad
family: acl4ssr
repository: https://github.com/ACL4SSR/ACL4SSR
ref: master
path: Clash/BanAD.list
canonical_url: https://raw.githubusercontent.com/ACL4SSR/ACL4SSR/master/Clash/BanAD.list
format: text
behavior: classical
target_policy: REJECT
order: 200
exclusive_group: ads.primary
mirror_kind: github-prefix
content_audit: text
license: CC-BY-SA-4.0
```

必须保留：

- 稳定 source ID；
- repository/ref/path；
- family、overlap/exclusive group；
- canonical URL 与 effective URL；
- format/behavior；
- target policy/order；
- license/attribution/provenance；
- resolved commit SHA 和内容 SHA-256。

## 3. 规则顺序

```text
0–39     用户精确放行、阻断与手动规则
40–79    精确遥测等本地安全规则
80–99    进程规则
100–199  LAN/private
200–299  广告、恶意、钓鱼
300–499  Claude/AI/流媒体/服务专用
500–699  宽泛代理域名
700–799  国内/直连域名
800–899  IP-CIDR/IP-ASN/GEOIP
900–999  最后地理兜底
1000     唯一 MATCH
```

服务专用规则必须位于宽泛地理规则和最终 MATCH 之前。不同 family 可以有内部顺序约束，编译器需要同时满足全局 order 和 family order。

## 4. 精确去重

归一化：

- matcher type 大写；
- 域名小写、去尾点、IDNA 规范化；
- CIDR 转为真实网络地址；
- flags 规范化排序；
- canonical source 不受镜像 URL 影响。

matcher、value、flags、target 全部相同才属于无争议重复。

## 5. 保守语义去重

仅在目标策略和 flags 相同时：

- `DOMAIN-SUFFIX,example.com` 可以覆盖同策略子域 `DOMAIN`；
- 大 CIDR 可以覆盖同策略、同 flags 的小 CIDR；
- 同一 provider 的 URL 变体只保留一个 canonical source；
- 被删除项仍保留 provenance 和获胜规则 ID。

## 6. 冲突而非重复

以下必须进入冲突报告：

- 相同 matcher/value 指向不同策略；
- 父后缀与子域规则指向不同策略；
- CIDR 包含关系的目标不同；
- 两个完整 routing family 同时启用；
- 多个 `ads.primary` 同时启用；
- 用户规则与远程规则目标不同。

优先级原则：

1. 用户显式规则；
2. 内置精确 Claude 规则；
3. 更具体 matcher；
4. 更高 source priority；
5. 仍不明确时要求用户确认。

## 7. MRS 边界

- 只有 MRS 时标记 `content_audit: opaque`；
- 可以按 source ID、canonical URL、provider 引用和文件 hash 去重；
- 逐条去重需要同源 text/yaml，或先用绑定版本 Mihomo 转换；
- 不在报告中声称已完成 opaque MRS 的内容级去重。

## 8. GitHub 镜像

```yaml
github_transport:
  proxy_url: https://v6.gh-proxy.org/
  apply_to:
    - raw.githubusercontent.com
    - github.com/*/releases/download/*
    - github.com/*/raw/*
  direct_fallback: true
```

非 GitHub 地址保持原样。下载后必须识别 HTML 错误页、限制大小、验证格式并计算 SHA-256。

## 9. 去重报告

- 输入、规范化、保留、精确重复数量；
- 同策略语义包含数量；
- 跨策略冲突数量；
- opaque provider 数量；
- 每条删除/遮蔽项的 source ID 和获胜规则；
- commit SHA、内容 SHA-256、获取时间；
- 缺失策略组、循环引用和唯一 MATCH 校验。
