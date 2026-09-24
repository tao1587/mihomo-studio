# Mihomo Studio 开源规则生态调研与内置 Catalog 建议

> 快照日期：2026-08-14（America/Los_Angeles）。热度、默认分支、许可证和 `pushed_at` 来自 GitHub 官方 REST API；规则结构、格式、上游关系来自各项目官方 README、仓库文件树和 Release。DivineEngine 已排除，不进入候选、依赖或兼容层。

## 1. 结论

1. **内置 Catalog 不应把所有热门规则同时启用。** MetaCubeX、Loyalsoldier、DustinWin、blackmatrix7、ACL4SSR 之间存在明显的上下游和内容重叠；“全部勾选”会制造大量重复和策略冲突。
2. 建议把规则源分成三层：
   - **默认组合**：ACL4SSR 的可审计文本规则 + Mihomo Studio 自带 Claude 规则包；需要地理数据时使用 MetaCubeX 的 Mihomo 原生资源。
   - **可替换规则家族**：Loyalsoldier、SukkaW、DustinWin。它们是成体系的替代方案，不与默认宽泛规则同时启用。
   - **完整模式的细粒度补充**：blackmatrix7 的单服务规则、anti-AD/AWAvenue 的广告规则、ACL4SSR 的单服务规则。
3. **简单模式默认采用可读的 `.list`/`.yaml`，而不是只提供二进制 `.mrs`。** 这样能做内容级去重、冲突解释和来源追踪。MRS 可以作为最终优化格式，但其审计输入必须有同源文本版本；仅有 MRS 时标记为 `opaque`，只做 provider 级去重。
4. 广告源必须互斥：
   - 保守：`ACL4SSR/BanAD`；
   - 平衡：`AWAvenue Only.Ads`；
   - 强力：`anti-AD` 或 AWAvenue 完整版。
   - anti-AD 明确聚合了 AWAvenue 和 ACL4SSR，三者同时启用没有意义。
5. GitHub 镜像应是**传输层属性**，不可改变 canonical identity。只对 GitHub Raw、GitHub Release、GitHub Blob/Raw 重定向地址应用 `proxy_url`；不可把 GitHub 前缀拼到 `ruleset.skk.moe` 等非 GitHub 地址。

## 2. 热度与维护快照

| 项目 | Stars | Forks | 最近 push（UTC） | 许可证 | 维护判断 |
|---|---:|---:|---|---|---|
| [Loyalsoldier/clash-rules](https://github.com/Loyalsoldier/clash-rules) | 28,007 | 2,181 | 2026-08-14 22:51 | GPL-3.0 | 活跃，每日构建 |
| [blackmatrix7/ios_rule_script](https://github.com/blackmatrix7/ios_rule_script) | 27,542 | 4,054 | 2026-08-13 19:00 | GPL-2.0 | 活跃，规则面最广 |
| [Loyalsoldier/v2ray-rules-dat](https://github.com/Loyalsoldier/v2ray-rules-dat) | 20,484 | 2,696 | 2026-08-14 22:16 | GPL-3.0 | 活跃，每日构建 |
| [privacy-protection-tools/anti-AD](https://github.com/privacy-protection-tools/anti-AD) | 10,647 | 790 | 2026-08-13 19:32 | MIT | 活跃，广告专项 |
| [v2fly/domain-list-community](https://github.com/v2fly/domain-list-community) | 9,334 | 1,399 | 2026-08-15 02:40 | MIT | 活跃，权威上游数据 |
| [ACL4SSR/ACL4SSR](https://github.com/ACL4SSR/ACL4SSR) | 6,507 | 2,017 | 2026-08-11 01:11 | CC-BY-SA-4.0 | 活跃，兼容 subconverter 与 Mihomo |
| [Loyalsoldier/geoip](https://github.com/Loyalsoldier/geoip) | 6,425 | 886 | 2026-08-13 23:02 | CC-BY-SA-4.0 | 活跃，每周构建 |
| [TG-Twilight/AWAvenue-Ads-Rule](https://github.com/TG-Twilight/AWAvenue-Ads-Rule) | 6,365 | 164 | 2026-07-26 18:07 | GPL-3.0 | 有维护，广告专项 |
| [MetaCubeX/meta-rules-dat](https://github.com/MetaCubeX/meta-rules-dat) | 4,948 | 759 | 2026-08-14 22:50 | GPL-3.0 | 活跃，Mihomo 原生 |
| [SukkaW/Surge](https://github.com/SukkaW/Surge) | 4,381 | 309 | 2026-08-14 12:22 | AGPL-3.0；单个中国 IP 文件 CC BY-SA 2.0 | 活跃、强约束、文档完善 |
| [DustinWin/ruleset_geodata](https://github.com/DustinWin/ruleset_geodata) | 1,363 | 125 | 2026-08-14 19:56 | GPL-3.0 | 活跃，每日构建 |
| [Repcz/Tool](https://github.com/Repcz/Tool) | 1,100 | 133 | 2026-08-14 06:13 | MIT | 活跃、CI 自动构建 |

GitHub API 证据入口示例：

- [ACL4SSR API](https://api.github.com/repos/ACL4SSR/ACL4SSR)
- [MetaCubeX API](https://api.github.com/repos/MetaCubeX/meta-rules-dat)
- [blackmatrix7 API](https://api.github.com/repos/blackmatrix7/ios_rule_script)
- [Loyalsoldier/clash-rules API](https://api.github.com/repos/Loyalsoldier/clash-rules)
- [SukkaW API](https://api.github.com/repos/SukkaW/Surge)

Stars/Forks 只用于生态热度排序，不能作为默认启用依据；格式可审计性、许可、上游重叠和失败模式更重要。

## 3. 项目逐项比较

### 3.1 ACL4SSR/ACL4SSR

- 官方仓库：[ACL4SSR/ACL4SSR](https://github.com/ACL4SSR/ACL4SSR)
- 许可证：CC BY-SA 4.0。
- 当前格式：
  - `Clash/*.list`、`Clash/Ruleset/*.list`：classical/text；
  - `Clash/Providers/**/*.yaml`：Clash provider YAML；
  - [`Clash/mrs`](https://github.com/ACL4SSR/ACL4SSR/tree/master/Clash/mrs)：Mihomo MRS，已覆盖 domain/ip 两类；
  - `Clash/config/*.ini`：subconverter 外部配置。
- 类别：LAN、广告、隐私、国内域名/IP、GFW/ProxyLite、下载、Apple、Microsoft、Google、流媒体、游戏、AI/OpenAI/Claude/Gemini 等。
- 优点：
  - 与 NAS 现有 subconverter 模型最接近；
  - 文本、YAML、MRS 三种形态同时存在，适合“文本审计、MRS 输出”；
  - `BanAD` 官方说明偏保守，适合简单模式。
- 风险：
  - EasyList、服务规则和其他聚合库存在大量交集；
  - CC BY-SA 要求保留署名和相同方式共享衍生数据；
  - `BanProgramAD`、EasyPrivacy 等可能引入功能副作用，不能在简单模式全部默认启用。
- 建议：**简单模式默认家族；完全模式提供完整目录。**

### 3.2 MetaCubeX/meta-rules-dat

- 官方仓库：[MetaCubeX/meta-rules-dat](https://github.com/MetaCubeX/meta-rules-dat)
- 许可证：GPL-3.0。
- Mihomo 规则位于 [`meta` 分支](https://github.com/MetaCubeX/meta-rules-dat/tree/meta)，主要提供 domain/ipcidr MRS；另有 geosite、geoip、mmdb、db、metadb 和 BundleMRS。
- 常见类别：private、cn、geolocation-!cn、gfw、openai、google、github、microsoft、netflix、telegram、youtube、proxymedia 等。
- 上游：Loyalsoldier、v2fly/domain-list-community、blackmatrix7、felixonmars、gfwlist、多种广告列表等。
- 优点：Mihomo 原生、体积小、加载快、更新活跃。
- 风险：
  - MRS 是二进制，生成器不能直接做可解释的逐条去重；
  - 与 Loyalsoldier、blackmatrix7、v2fly 派生集高度重叠；
  - README 明确说明其常用 ruleset 集合来自 blackmatrix7。
- 建议：
  - 作为默认地理数据库和“原生 MRS 模式”；
  - 简单模式如果强调去重报告，应改用同类文本源，或明确显示“内容级审计不可用”；
  - 不与 Loyalsoldier/DustinWin 的宽泛集合同时启用。

### 3.3 blackmatrix7/ios_rule_script

- 官方仓库：[blackmatrix7/ios_rule_script](https://github.com/blackmatrix7/ios_rule_script)
- 许可证：GPL-2.0。
- Clash 目录当前主要是 `.list` 和 `.yaml`，有 classical、domain、IP、No_Resolve 变体；规则类别超过数百种，涵盖具体网站、AI、流媒体、游戏、下载、金融、社交、广告等。
- 官方 README 明确称“并不生产规则，只是开源规则的搬运工”，每个服务目录通常记录来源。
- 优点：细粒度覆盖最丰富，特别适合完全模式按服务开关。
- 风险：
  - 本身是大型聚合层，与所有综合库交叉；
  - 上游许可和来源需要随单个目录审查；
  - 规则体量大，全部启用会拖慢编译与匹配；
  - 不应把整个仓库作为一个总规则包。
- 建议：**只作为完全模式的“单服务补充库”，默认全部关闭。** Claude 应优先使用 Mihomo Studio 自带的受控 Claude 规则包，而不是直接依赖其聚合列表。

### 3.4 Loyalsoldier/clash-rules

- 官方仓库：[Loyalsoldier/clash-rules](https://github.com/Loyalsoldier/clash-rules)
- 许可证：GPL-3.0；北京时间每日 06:30 自动构建。
- 格式：`release` 分支的 text 规则；behavior 分为 domain、ipcidr、classical。
- 类别：private、reject、proxy、direct、gfw、tld-not-cn、apple、icloud、google、telegramcidr、lancidr、cncidr、applications。
- 上游：Loyalsoldier/v2ray-rules-dat、v2fly/domain-list-community、felixonmars/dnsmasq-china-list、17mon/china_ip_list。
- 优点：规则家族小而完整、文本可审计、README 直接给出 Clash Verge Rev/Mihomo 用法。
- 风险：与 MetaCubeX 和 v2ray-rules-dat 近乎同源；服务细分不如 blackmatrix7。
- 建议：**作为 ACL4SSR 之外的“核心文本规则家族”替代项。** 不能和 MetaCubeX/DustinWin 宽泛集合一起默认启用。

### 3.5 Loyalsoldier/v2ray-rules-dat 与 Loyalsoldier/geoip

- 官方仓库：[v2ray-rules-dat](https://github.com/Loyalsoldier/v2ray-rules-dat)、[geoip](https://github.com/Loyalsoldier/geoip)
- 格式：dat、mmdb、MRS、Clash/Surge text 等；前者每日构建，后者每周构建。
- 适合：geosite/geoip 模式、国家/地区和 ASN/IP 资源、自定义地理数据。
- 风险：与 MetaCubeX 数据链直接重叠；不适合作为另一个“服务规则目录”同时叠加。
- 建议：**完全模式的高级地理资源替代项，默认关闭。**

### 3.6 SukkaW/Surge

- 官方仓库：[SukkaW/Surge](https://github.com/SukkaW/Surge)
- 许可证：主体 AGPL-3.0；`List/ip/china_ip.conf` 为 CC BY-SA 2.0。
- Mihomo 格式：
  - `/Clash/domainset/`：`behavior: domain, format: text`；
  - `/Clash/non_ip/`：`behavior: classical, format: text`；
  - `/Clash/ip/`：`behavior: classical` 或 `ipcidr`，`format: text`。
- 类别：广告/隐私/恶意/钓鱼、CDN、分地区流媒体、AI、Telegram、Apple/Microsoft/CDN、下载、LAN、chnroute 等。
- 官方强制顺序：`domainset -> non_ip -> ip`，所有域名类规则必须在 IP 类规则前。
- 优点：生成逻辑公开、白名单和顺序说明非常清楚、规则策略完整。
- 风险：
  - 是一套强约束的完整体系，不适合零散地和其他综合库拼接；
  - 默认资产域名是 `ruleset.skk.moe`，GitHub `proxy_url` 不能直接套用；
  - AGPL 数据/生成物的分发边界需要保留许可和署名。
- 建议：**完全模式的独立“专家规则家族”，默认关闭。** 启用后由编译器锁定它要求的相对顺序。

### 3.7 DustinWin/ruleset_geodata

- 官方仓库：[DustinWin/ruleset_geodata](https://github.com/DustinWin/ruleset_geodata)
- 许可证：GPL-3.0；每天 03:00（UTC+8）构建。
- 格式：Mihomo `.list`/`.mrs`，以及 geosite/geoip/dat/mmdb/metadb；资产通过 GitHub Release 发布。
- 类别：fakeip-filter、ads、private、applications、AI、流媒体、游戏、CN/proxy/GFW、private/CN/Telegram/Netflix IP 等。
- 上游：DustinWin 自有 domain/geoip 构建链，同时聚合 v2fly、blackmatrix7、ACL4SSR AI、anti-AD、Loyalsoldier 等。
- 优点：格式完整、Mihomo 优先、类别覆盖均衡、适合作为整套预设。
- 风险：它已经是“聚合后的聚合”；再叠加 ACL4SSR、blackmatrix7、Loyalsoldier 或 MetaCubeX 会大幅重复。
- 建议：**作为完整的替代预设包，不作为默认源的补充。**

### 3.8 Repcz/Tool

- 官方仓库：[Repcz/Tool](https://github.com/Repcz/Tool)
- 许可证：MIT；输出在 `X` 分支；Mihomo 使用 classical/text `.list`。
- 类别：Reject、AI、Telegram、Twitter、Facebook、TikTok、游戏、Google、Microsoft、GitHub、流媒体、Proxy、AppleProxy、ChinaDomain/ChinaIP 等。
- 优点：结构简单、跨 Mihomo/Surge/sing-box、CI 持续更新。
- 风险：官方 README 对逐项上游和生成规则解释不足；`X` 是可变输出分支。
- 建议：**完全模式实验源，默认关闭；Catalog 标注 `provenance: partial`。**

### 3.9 anti-AD 与 AWAvenue

#### anti-AD

- 官方仓库：[privacy-protection-tools/anti-AD](https://github.com/privacy-protection-tools/anti-AD)
- MIT；支持 `anti-ad-clash.yaml`、domain text 和 Mihomo MRS。
- 它聚合、去重并抽象多个广告上游，README 明确致谢 AWAvenue、ACL4SSR、AdGuard、EasyList 等。
- 适合强力广告拦截；争议域名可能影响业务，需要白名单覆盖。

#### AWAvenue

- 官方仓库：[TG-Twilight/AWAvenue-Ads-Rule](https://github.com/TG-Twilight/AWAvenue-Ads-Rule)
- GPL-3.0；提供 Clash classical/domain YAML 与 MRS，并分为 `Only.Ads`、`No.Privacy`、`No.Unwelcome` 和完整版本。
- `Only.Ads` 特别适合“只拦广告、不顺带屏蔽隐私/不受欢迎内容”的简单交互。

#### 组合规则

- anti-AD 与 AWAvenue、ACL4SSR 广告规则必须属于同一 `exclusive_group: ads.primary`。
- 完全模式允许用户再添加自定义白名单，但不允许同时勾选多个主广告源而不显示阻断性警告。

### 3.10 v2fly/domain-list-community

- 官方仓库：[v2fly/domain-list-community](https://github.com/v2fly/domain-list-community)
- MIT；社区管理、非策略化，输出 geosite 数据；规则有 include 和 `@cn/@ads` 属性语义。
- 它是 MetaCubeX、Loyalsoldier、DustinWin 等的核心上游之一。
- 建议：Catalog 把它标成 `upstream-data`，不作为普通用户可直接勾选的 Clash rule-provider；以后实现 v2fly 语法编译器后再进入专家模式。

## 4. 上下游与重叠关系

| 选择 A | 同时选择 B | 风险 | Catalog 行为 |
|---|---|---|---|
| MetaCubeX 宽泛 geosite/geoip | Loyalsoldier 宽泛规则 | 高度同源 | 阻止简单模式同时启用 |
| MetaCubeX | blackmatrix7 宽泛服务集合 | Meta 已聚合 blackmatrix7 常用集合 | 只允许单个细分服务覆盖 |
| DustinWin | ACL4SSR/blackmatrix7/Loyalsoldier/MetaCubeX | DustinWin 已聚合它们 | 标为替代规则家族 |
| Loyalsoldier clash-rules | v2ray-rules-dat | clash-rules 主要从后者生成 | 二选一 |
| anti-AD | AWAvenue 或 ACL4SSR 广告 | anti-AD 已聚合二者 | `ads.primary` 互斥 |
| SukkaW 完整家族 | 其他宽泛 family | 匹配顺序和覆盖关系冲突 | 专家预设独占 |
| ACL4SSR AI | blackmatrix AI / Meta OpenAI / Dustin AI | 服务域名交叉 | 允许，但先预览冲突和重复 |
| 本地 Claude 包 | 任意 Claude 聚合规则 | 本地规则更精准、版本可控 | 本地规则最高优先；外部 Claude 默认关闭 |

## 5. Catalog 数据模型

Catalog 应存 repo identity，而不是只存一个 URL：

```yaml
schema_version: 1
sources:
  - id: acl4ssr.ban-ad
    title: ACL4SSR 保守广告
    family: acl4ssr
    overlap_groups: [ads.primary, acl4ssr.general]
    repository: https://github.com/ACL4SSR/ACL4SSR
    ref: master
    audit_path: Clash/BanAD.list
    optimized_path: Clash/mrs/BanAD_domain.mrs
    canonical_url: https://raw.githubusercontent.com/ACL4SSR/ACL4SSR/master/Clash/BanAD.list
    format: text
    behavior: classical
    target_policy: REJECT
    order: 200
    license: CC-BY-SA-4.0
    attribution_required: true
    mirror:
      kind: github-prefix
      allowed: true
    pin_strategy: resolve-commit-on-refresh
    defaults:
      simple: true
      full: true
```

必须字段：

- `id`：永久稳定，路径变化不能改 ID；
- `family`：用于整包替换；
- `overlap_groups` / `exclusive_group`：用于 UI 冲突；
- `repository + ref + path`：canonical identity；
- `audit_path`：可读输入；
- `optimized_path`：可选 MRS 输出；
- `format/behavior`；
- `target_policy` 与 `order`；
- `license`、`attribution_required`、`provenance`；
- `mirror.kind`；
- `pin_strategy`、最后解析 commit SHA、内容 SHA-256。

## 6. 可落地的内置 Source Catalog

下表的 `order` 越小越先匹配。`AI`、`GLOBAL`、`DOWNLOAD` 是逻辑策略名，编译时映射到用户实际策略组。

### 6.1 简单模式默认启用

| Source ID | Canonical URL / identity | behavior/format | 策略 | order | 默认 | 镜像 |
|---|---|---|---|---:|---|---|
| `acl4ssr.local-area-network` | `ACL4SSR/ACL4SSR@master:Clash/LocalAreaNetwork.list` | classical/text | DIRECT | 100 | 开 | GitHub prefix |
| `acl4ssr.ban-ad` | `ACL4SSR/ACL4SSR@master:Clash/BanAD.list` | classical/text | REJECT | 200 | 开 | GitHub prefix |
| `builtin.claude.route` | Mihomo Studio 内置版本化规则 | classical/inline | AI | 300 | 开 | 不适用 |
| `acl4ssr.download` | `ACL4SSR/ACL4SSR@master:Clash/Download.list` | classical/text | DOWNLOAD；简单模式映射 DIRECT | 350 | 开 | GitHub prefix |
| `acl4ssr.proxy-lite` | `ACL4SSR/ACL4SSR@master:Clash/ProxyLite.list` | classical/text | GLOBAL | 600 | 开 | GitHub prefix |
| `acl4ssr.china-domain` | `ACL4SSR/ACL4SSR@master:Clash/ChinaDomain.list` | classical/text | DIRECT | 700 | 开 | GitHub prefix |
| `acl4ssr.china-company-ip` | `ACL4SSR/ACL4SSR@master:Clash/ChinaCompanyIp.list` | classical/text | DIRECT/no-resolve | 850 | 开 | GitHub prefix |
| `builtin.geoip-cn` | Mihomo `GEOIP,CN` | inline | DIRECT/no-resolve | 900 | 开 | 不适用 |
| `builtin.match` | Mihomo `MATCH` | inline | 用户选择，默认 GLOBAL | 1000 | 开 | 不适用 |

说明：

- 默认组合延续 NAS 的 ACL4SSR 使用习惯，但去掉 DivineEngine 和硬编码节点名。
- `MATCH` 必须唯一且最后。
- 内置 Claude 包置于宽泛代理/直连规则之前；其遥测阻断、进程规则、UDP fail-closed 由独立开关控制。
- MetaCubeX 默认作为可选的 `geodata asset provider`，不和上表文本规则强行混合：
  - `metacubex.country-lite-mmdb`
  - `metacubex.geosite-lite-db`
  - `metacubex.asn-mmdb`

### 6.2 简单模式广告强度选项（互斥）

| Source ID | Canonical URL | 策略 | 默认 | 说明 |
|---|---|---|---|---|
| `acl4ssr.ban-ad` | `https://raw.githubusercontent.com/ACL4SSR/ACL4SSR/master/Clash/BanAD.list` | REJECT | 保守默认 | 小、风险低 |
| `awavenue.ads-only` | `https://raw.githubusercontent.com/TG-Twilight/AWAvenue-Ads-Rule/main/Filters/AWAvenue-Ads-Rule-Clash-Only.Ads.yaml` | REJECT | 平衡可选 | 只含广告，排除额外隐私/不受欢迎项 |
| `antiad.standard` | `https://raw.githubusercontent.com/privacy-protection-tools/anti-AD/master/anti-ad-clash.yaml` | REJECT | 强力可选 | 聚合广，必须支持白名单和误杀回滚 |

三者均设置 `exclusive_group: ads.primary`，切换时替换而不是叠加。

### 6.3 完全模式内置、默认关闭

| Source ID / family | Canonical repo/url | 建议策略 | order 范围 | 镜像 | 备注 |
|---|---|---|---:|---|---|
| `acl4ssr.ai` | `ACL4SSR/.../Clash/Ruleset/AI.list` | AI | 310 | 是 | 泛 AI；本地 Claude 仍更早 |
| `acl4ssr.openai` | `ACL4SSR/.../Clash/Ruleset/OpenAi.list` | AI | 320 | 是 | 单服务 |
| `acl4ssr.claude` | `ACL4SSR/.../Clash/Ruleset/Claude.list` | AI | 330 | 是 | 默认关，避免覆盖本地 Claude 包 |
| `blackmatrix.service.*` | `blackmatrix7/ios_rule_script@master:rule/Clash/<Service>/<Service>.list` | 用户策略 | 300–499 | 是 | 只按服务启用 |
| `loyalsoldier.clash.*` | `Loyalsoldier/clash-rules@release:<name>.txt` | direct/proxy/reject | 100–900 | 是 | 替代核心 family |
| `metacubex.geosite.*` | `MetaCubeX/meta-rules-dat@meta:geo/geosite/<name>.mrs` | 用户策略 | 300–799 | 是 | opaque/MRS；provider 级去重 |
| `metacubex.geoip.*` | `MetaCubeX/meta-rules-dat@meta:geo/geoip/<name>.mrs` | 用户策略 | 800–899 | 是 | IP 必须位于 domain 后 |
| `sukkaw.*` | `https://ruleset.skk.moe/Clash/...` | 按官方模板 | 固定相对顺序 | 否 | 专家家族，domainset→non_ip→ip |
| `dustinwin.*` | `DustinWin/ruleset_geodata` GitHub Release | 按资产 | 固定模板 | 是 | 替代 family，不和 broad 源叠加 |
| `repcz.*` | `Repcz/Tool@X:mihomo/Rules/<name>.list` | 用户策略 | 300–899 | 是 | 实验；provenance partial |
| `loyalsoldier.geoip.*` | `Loyalsoldier/geoip@release` | DIRECT/代理 | 800–899 | 是 | 高级地理资源 |

## 7. 推荐的规则顺序

```text
0–39     用户精确放行/阻断、手动规则
40–79    精确遥测阻断等本地安全规则
80–99    进程规则
100–199  LAN/private 域名规则
200–299  广告、恶意、钓鱼
300–499  Claude/AI/流媒体/服务专用规则
500–699  宽泛代理域名
700–799  国内/直连域名
800–899  IP-CIDR/IP-ASN/GEOIP，通常带 no-resolve
900–999  最后地理兜底
1000     唯一 MATCH
```

Sukka family 启用时，还必须满足其官方顺序：`domainset -> non_ip -> ip`。

## 8. 去重与冲突处理建议

### 8.1 可安全自动执行

1. 格式归一：类型大写、域名小写、去尾点、IDNA 规范化、CIDR 网络地址归一、`no-resolve` 排序。
2. 同类型、同值、同参数、同策略的精确去重。
3. 同策略下，`DOMAIN-SUFFIX,example.com` 可覆盖其子域的 `DOMAIN`；保留来源清单。
4. 同策略下，较宽 CIDR 可覆盖完全包含的较窄 CIDR。
5. 同一 source ID 的 URL 变体、镜像 URL、Raw/Release URL只算一个 canonical source。

### 8.2 不应静默执行

1. 不用 `DOMAIN-KEYWORD` 自动删除 DOMAIN/DOMAIN-SUFFIX；关键字过宽。
2. 同 matcher 指向不同策略是冲突，不是重复；按优先级保留胜者并展示被遮蔽项。
3. 不跨不同策略做 suffix/CIDR 包含删除；较窄规则可能是例外。
4. 不对 MRS 声称完成内容级去重；只做文件哈希、source ID 和 provider 引用去重。
5. 不把“来自同一上游”直接当作内容相同；以规范化后的规则和 SHA-256 为准。

### 8.3 报告字段

每次生成应输出：

- 输入规则数、规范化数、精确重复数、语义包含数；
- 同策略删除数、跨策略冲突数、opaque provider 数；
- 每条删除/遮蔽的 source ID、原始行、获胜规则；
- source commit SHA、内容 SHA-256、抓取时间、实际传输 URL（脱敏）；
- 唯一 MATCH、缺失策略组引用、循环策略组等结构校验。

## 9. 镜像设计

```yaml
github_transport:
  proxy_url: https://v6.gh-proxy.org/
  apply_to:
    - raw.githubusercontent.com
    - github.com/*/releases/download/*
    - github.com/*/raw/*
  direct_fallback: true
```

规则：

1. Catalog 永远保存 canonical repo/ref/path；镜像只生成 effective fetch URL。
2. 镜像 URL 必须规范化尾斜线，防止双斜线或路径拼错。
3. 不对 `cdn.jsdelivr.net`、`ruleset.skk.moe`、`anti-ad.net` 自动加 GitHub 前缀。
4. 下载后校验期望格式、最大体积、SHA-256；HTML 错误页不可当规则成功写入。
5. 记录直接源和镜像源的结果；如果镜像返回旧内容，通过 commit/hash 提示用户。

## 10. 不建议进入内置 Catalog 的仓库

以下仓库可以供用户自定义 URL 导入，但当前不作为内置可信源：

| 项目 | 快照热度 | 原因 |
|---|---|---|
| [dler-io/Rules](https://github.com/dler-io/Rules) | 1,448 stars / 376 forks | GitHub API 未识别根许可证；在许可明确前不内置 |
| [qichiyuhub/rule](https://github.com/qichiyuhub/rule) | 1,492 / 1,296 | 配置聚合仓库，GitHub API 未识别根许可证，非 canonical data source |
| [LM-Firefly/Rules](https://github.com/LM-Firefly/Rules) | 751 / 120 | 自用/备份定位，GitHub API 未识别根许可证 |
| [HenryChiao/MIHOMO_YAMLS](https://github.com/HenryChiao/MIHOMO_YAMLS) | 2,550 / 242 | 配置模板聚合，适合 UI/预设参考，不是 canonical 规则源 |

“API 未识别许可证”不等同于断言作者没有任何许可声明，但不足以满足默认内置源的自动许可审计要求。

## 11. 对 Mihomo Studio 的最终建议

### 简单模式

- 只显示：节点/订阅、广告强度、AI/Claude、国内直连、默认兜底。
- 默认规则 family：ACL4SSR 可读文本；默认广告：BanAD。
- 默认只启用一个宽泛 family、一个广告源、本地 Claude 包。
- 自动生成唯一 MATCH 和实际存在的策略组。

### 完全模式

- 显示 source family、单服务 catalog、镜像、许可证、上游关系、优先级、opaque 状态。
- blackmatrix7 用作服务目录；SukkaW、Loyalsoldier、DustinWin 是替代家族。
- 支持切换 `audit text` 与 `optimized MRS`，但输出必须解释去重层级。

### 第一版应内置的仓库级 Catalog

1. ACL4SSR：默认和完整目录；
2. MetaCubeX/meta-rules-dat：地理资产和 MRS 专家模式；
3. blackmatrix7：完全模式服务目录；
4. Loyalsoldier/clash-rules：替代核心文本 family；
5. Loyalsoldier/geoip：高级地理 family；
6. SukkaW/Surge：专家独立 family；
7. DustinWin/ruleset_geodata：完整替代 family；
8. anti-AD 与 AWAvenue：互斥广告选项；
9. Repcz/Tool：实验目录，默认关闭；
10. v2fly/domain-list-community：只登记为 upstream-data。

## 12. 官方证据链接

- [ACL4SSR README](https://github.com/ACL4SSR/ACL4SSR/blob/master/README.md)
- [ACL4SSR Clash/MRS](https://github.com/ACL4SSR/ACL4SSR/tree/master/Clash/mrs)
- [MetaCubeX meta-rules-dat README](https://github.com/MetaCubeX/meta-rules-dat/blob/master/README.md)
- [MetaCubeX Mihomo meta branch](https://github.com/MetaCubeX/meta-rules-dat/tree/meta)
- [blackmatrix7 README](https://github.com/blackmatrix7/ios_rule_script/blob/master/README.md)
- [blackmatrix7 Clash rules](https://github.com/blackmatrix7/ios_rule_script/tree/master/rule/Clash)
- [Loyalsoldier clash-rules README](https://github.com/Loyalsoldier/clash-rules/blob/master/README.md)
- [Loyalsoldier v2ray-rules-dat README](https://github.com/Loyalsoldier/v2ray-rules-dat/blob/master/README.md)
- [Loyalsoldier geoip README](https://github.com/Loyalsoldier/geoip/blob/master/README.md)
- [SukkaW README](https://github.com/SukkaW/Surge/blob/master/README.md)
- [DustinWin ruleset_geodata README](https://github.com/DustinWin/ruleset_geodata/blob/main/README.md)
- [Repcz Tool README](https://github.com/Repcz/Tool/blob/X/README.md)
- [anti-AD README](https://github.com/privacy-protection-tools/anti-AD/blob/master/README.md)
- [AWAvenue README](https://github.com/TG-Twilight/AWAvenue-Ads-Rule/blob/main/README.md)
- [v2fly domain-list-community README](https://github.com/v2fly/domain-list-community/blob/master/README.md)
