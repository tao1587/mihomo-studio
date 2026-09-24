# NAS subconverter 参考实现审计

## 1. 只读检查范围

位置：`/home/lol/apps/subconverter`

当前形态：

- Docker 化的 subconverter 服务；
- 对外端口 12000；
- 内部静态订阅服务；
- `base-lite` / `base-full` 模板；
- 多订阅合并脚本；
- 规则更新与去重脚本；
- ACL4SSR 以及多套历史第三方规则目录；
- GitHub 镜像配置；
- 自定义节点和动态国家组。

本审计没有复制或记录 NAS 中的订阅 token、节点凭据或 API token。

## 2. 值得沿用的思路

### 多种 User-Agent 和格式探测

订阅端点可能根据 User-Agent 返回 YAML、Base64、URI 或错误页。新应用保留“多 UA 探测 + 内容识别”，但将秘密响应限制在内存和受控缓存。

### 来源标签

NAS 会给节点名增加来源标签，便于区分同名节点。新应用保留此能力，并改为稳定来源 ID + 可编辑显示前缀。

### 节点名先解码再识别国家

VLESS/Trojan 名称可能是 percent-encoded。新应用在国家识别前做 URL decode、Unicode 规范化和 emoji/别名分析。

### 动态国家组

NAS 只为当前存在的国家生成组，并在订阅更新时清理失效组。该行为适合作为简单模式默认值。

### Lite / Full 两套预设

NAS 的 `base-lite` 与 `base-full` 对应本项目的“简单模式”和“完全模式”，但新项目把差异建模成结构化配方，不再维护多份容易漂移的 INI 文本。

### 规则源顺序优先

NAS 按模板中的来源顺序决定跨文件去重优先级。新应用保留“先定义优先级，再去重”的原则，并增加冲突可见性和语义边界。

## 3. 不直接复制的问题

### 多 YAML 来源覆盖

当前合并脚本对多个 YAML 来源不是结构化合并，后处理的 YAML 可能覆盖前一个。新应用必须把每个来源解析为 Node IR 后再合并。

### 节点只按整行去重

当前 Base64 节点按原始整行去重，名称变化会掩盖同一节点，参数顺序变化也会导致重复。新应用使用规范化内容指纹，且不会把同端点不同凭据静默合并。

### 规则去重实际只覆盖 ACL4SSR 目录

NAS 虽然保存多套历史第三方目录，但实际去重循环集中在 ACL4SSR；当前 lite/full 模板也主要激活 ACL4SSR。新应用只处理 catalog 中明确启用且仍可审计的来源。

### `DOMAIN-KEYWORD` 自动覆盖过宽

当前脚本会把被 keyword 覆盖的域名规则自动注释。keyword 可能误覆盖有意的精确例外。新应用默认只报告该关系，不自动删除。

### 硬编码节点名

当前 AI 策略依赖一个硬编码节点名。新应用使用“AI 出口角色”或用户选择的策略组，不把具体节点名称写进内置模板。

### 凭据和权限

当前脚本会在输出和链接中拼接 token/订阅地址，部分生成文件权限较宽。新应用：

- 日志不写秘密 URL/token；
- metadata 与 secret 分离；
- 临时文件最小权限；
- 导出 YAML 为当前用户私有；
- 生成报告不含 query/token；
- UI 中默认遮罩。

### 常驻本地 HTTP 拼装

NAS 通过 loopback 静态服务器把本地订阅交给 subconverter。桌面应用不需要用 URL 串联内部数据，改用内存中的结构化对象和短生命周期 sidecar。

### 规则源配置未完全数据化

当前 `config.json` 的统一 rules 配置没有实际启用，规则来源主要从模板文本抽取。新应用使用显式 RuleSource Registry，UI、更新器、去重和编译器读取同一份模型。

## 4. 迁移映射

| NAS 能力 | 新应用模块 |
|---|---|
| `update-subs.sh` | SourceService + Node parsers |
| UA 重试 | FetchPolicy |
| 来源标签 | Node provenance |
| `generate_dynamic_country_groups` | RegionClassifier + GroupCompiler |
| `base-lite.ini` | Simple Recipe |
| `base-full.ini` | Full Recipe |
| `update-rules.sh` | RuleCatalog + Rule IR + DedupEngine |
| `preprocessor.js` | Rust Node adapter / Mihomo resolver |
| subconverter HTTP API | Optional legacy sidecar adapter |
| 静态服务器 | 不再需要；内部对象直接传递 |

## 5. 结论

NAS 实现适合作为需求和迁移样本，不适合作为 Tauri 应用核心直接打包。推荐复用其“多来源、来源标签、动态地区、Lite/Full、规则顺序”产品经验，重新实现结构化节点/规则管线、安全存储、全来源去重和 Mihomo 验证。
