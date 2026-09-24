# Capability: 桌面体验

| 字段 | 值 |
|---|---|
| ID | `desktop-experience` |
| 状态 | `partial` |
| 前端 | React 19 + TypeScript + Vite |

## Purpose

提供从添加来源、选择规则到检查草案的低认知负担工作流，同时准确表达每个能力状态和隐私边界。

## Current State

- 居中单列三步工作区：节点来源、模式与规则、生成与结果；不保留常驻左右侧栏；
- 支持订阅链接与直接节点文本，文件导入尚未接通；
- 支持简单/完全模式、规则来源选择和 GitHub 镜像输入；
- React 已按 `app / features / shared` 组织；
- Tauri 运行时通过 shared API adapter 调用 Rust IPC；浏览器模式使用静态 catalog 和本地草案预览；
- 默认折叠的编译计划显示策略组、规则顺序，以及代理 DNS、DNS 劫持、TUN strict route、IPv6、DIRECT、bootstrap 和 provider 更新约束；
- Tauri 运行时可生成经过静态回读的严格隐私 YAML，并以默认折叠的敏感区域显示和复制；
- VLESS URI/Base64 VLESS 来源卡显示“可转换并编译”；未映射协议不再误报“可进入编译”；
- VLESS 参数级预检失败的来源卡显示“参数预检未通过”和脱敏汇总，不再显示“可转换并编译”；
- 域名节点可通过 `source:node=IPv4` 编辑器提交用户确认的 bootstrap 映射；来源移除时清空位置映射，避免错配；
- 顶栏提供“配置生成”和“节点转换”两个一级入口；节点转换拥有独立页面、协议感知输入和结果状态，支持 VLESS 与单 Peer WireGuard 客户端文本，不嵌入三步配置工作流；
- 独立节点转换在单个 IP 服务器节点成功后可额外展示内置单节点 Clash 模版的完整 YAML；节点名和策略组引用使用该 IP，完整结果可复制并提供建议文件名；
- 生成中的进度与失败原因固定显示在主操作附近，并通过 `status`/`alert` 语义播报；
- 浏览器模式仍只生成方案预览；当前没有应用内文件导出、diff、用户模版保存或外部应用同步。

## Requirements

### Requirement: 单列三步主流程

首要工作流 SHALL 以“添加节点来源 → 选择模式与规则 → 生成与结果”为主线，并使用居中单列布局。桌面界面不得保留只用于重复步骤标题的左侧导航，也不得用常驻右侧栏展示大段草案说明。未实现的步骤必须明确标记，不得用可点击外观暗示已经接通。

#### Scenario: Tauri 严格编译

- **WHEN** 用户在 Tauri 运行时点击“生成严格隐私 YAML”
- **THEN** UI 先更新草案，再调用严格编译 IPC
- **AND** 静态检查成功后显示“YAML 已生成”和无秘密数量/hash 报告
- **AND** YAML 默认折叠并标记为敏感内容。

### Requirement: 生成与结果聚焦

第三步 SHALL 默认直接显示生成状态和唯一主操作。策略组、规则顺序、隐私硬约束与草案警告 SHALL 收进默认折叠的“编译计划”；编译结果、敏感 YAML 展开和复制操作 SHALL 在同一区域按需出现。

#### Scenario: 尚未生成

- **WHEN** 用户滚动到第三步
- **THEN** “生成严格隐私 YAML”按钮无需展开说明即可操作
- **AND** 编译计划默认折叠。

#### Scenario: 已生成

- **WHEN** 严格编译和静态回读成功
- **THEN** 第三步显示无秘密报告
- **AND** 敏感 YAML 仍需用户主动展开。

### Requirement: 简单与完全模式

简单模式 SHALL 只暴露常用意图和推荐默认；完全模式 MAY 展开规则 family、优先级、DNS/TUN、去重和输出细节。两种模式必须共享同一权威领域不变量。

#### Scenario: 模式切换

- **WHEN** 用户从简单模式切换到完全模式
- **THEN** 规则来源可见范围和默认选择按 catalog 更新
- **AND** 已选择但在新模式非法的互斥项不得被静默保留

### Requirement: 秘密输入

订阅 URL SHALL 默认遮罩。UI 通知、来源卡片、错误区域和调试输出不得回显 path、query、fragment 或节点凭据。

#### Scenario: 订阅探测成功

- **WHEN** UI 收到 `SourceInspectionSummary`
- **THEN** 只显示 safe label、格式、数量、协议、警告和 resolver 状态
- **AND** 单独显示当前格式是否已接通编译

### Requirement: 多节点输入可读性

直接节点输入区 SHALL 使用完整内容宽度和多行编辑高度。编辑器 SHALL 保留显式换行，不得用软换行混淆 URI 边界，并 SHALL 显示非空输入行数量。

#### Scenario: 粘贴多个节点 URI

- **WHEN** 用户在直接节点输入区粘贴多个以换行分隔的 URI
- **THEN** 每个 URI 保持在独立视觉行
- **AND** 长 URI 可横向滚动，编辑区可纵向调整
- **AND** 界面显示忽略空行后的输入行数量

### Requirement: 独立节点转换功能入口

应用 SHALL 在顶栏提供“配置生成”和“节点转换”两个一级功能入口。节点转换 SHALL 进入独立页面并拥有独立输入、转换状态、错误和敏感 YAML 结果；不得嵌入“直接粘贴节点”卡片，也不得复用配置生成来源输入状态。输入提示 SHALL 明确支持单条/多行/Base64 VLESS，以及一份 WireGuard `[Interface]` / `[Peer]` 客户端配置；用户不需要预先选择协议。

转换页面 SHALL 不显示三步配置工作流、规则来源、bootstrap、草案或严格配置生成操作。转换结果 SHALL 显示输出节点数和去重数；用户主动转换成功后，敏感 YAML SHALL 在当前独立页面直接可见并提供显式复制操作，不得放入默认关闭或可再次隐藏的 disclosure。只有无秘密成功摘要可进入 live-region，完整 YAML 不得被辅助技术作为状态更新自动播报。基础结果是 `proxies:` 片段；仅当单节点服务器为 IP literal 时，页面 MAY 提供去敏内置单节点模版组成的完整 YAML，并明确它沿用原模版设置且未经严格隐私或 Mihomo 运行验证。页面不得表述成已导入或已连接。

该常显规则只适用于独立节点转换结果；完整严格配置 YAML、订阅 URL 和其他秘密界面仍遵守既有折叠或遮罩规则。

#### Scenario: 从一级入口进入转换器

- **WHEN** 用户点击顶栏“节点转换”
- **THEN** 主内容切换为独立转换页面
- **AND** 页面不显示步骤 1/2/3、规则来源或 bootstrap 输入
- **AND** “直接粘贴节点”卡片不再显示转换按钮。

#### Scenario: 返回配置生成

- **GIVEN** 配置生成工作区已有来源或规则选择
- **WHEN** 用户进入转换器后返回“配置生成”
- **THEN** 配置生成状态保持不变
- **AND** 转换输入和结果不会加入节点来源。

#### Scenario: 编辑转换输入

- **WHEN** 用户在已有转换结果后修改独立转换页输入
- **THEN** 旧结果与旧错误立即清除
- **AND** 不把过期 YAML 继续显示为当前输入的结果。

#### Scenario: WireGuard 转换结果

- **WHEN** 用户粘贴有效 WireGuard 客户端文本并主动转换
- **THEN** 页面显示一个已生成节点的无秘密摘要
- **AND** 完整敏感 YAML 无需再次展开即可见
- **AND** 页面不声称已导入、已连接或已验证出口。

#### Scenario: 单个 IP 节点使用内置模版

- **WHEN** 用户转换后得到一个服务器为 IP literal 的节点
- **THEN** 结果页提供节点片段与内置模版完整 YAML 切换
- **AND** 完整 YAML 的唯一节点名、策略组节点引用和建议文件名使用该 IP
- **AND** 用户可主动复制完整 YAML，敏感内容不进入状态播报。

#### Scenario: 域名或多节点转换

- **WHEN** 结果有多个节点或单节点服务器不是 IP literal
- **THEN** 仍可查看和复制 `proxies:` 片段
- **AND** 页面说明完整模版当前不可用，不自动解析域名。

### Requirement: 事实状态文案

UI SHALL 使用与能力证据一致的状态词。serializer 与静态检查成功后允许“YAML 已生成”；绑定内核、文件和外部运行态证据完成前不得显示“Mihomo 已验证”“已导出”“已加载”或“出口已验证”。

### Requirement: 隐私约束预览

草案预览 SHALL 以“编译硬约束”显示严格隐私摘要，包括代理 DNS、DNS 劫持、TUN strict route、IPv6 关闭、禁止 DIRECT、受保护 bootstrap 和 provider 更新边界。

#### Scenario: 仅有隐私草案

- **WHEN** UI 展示严格隐私摘要
- **THEN** 文案说明这些字段是编译硬约束
- **AND** 不显示“DNS 已防泄漏”“原始 IP 已隐藏”或其他运行态成功结论。

### Requirement: 浏览器预览边界

非 Tauri 环境 MAY 提供 UI 预览，但 SHALL 明确显示“浏览器界面预览”。涉及网络抓取、秘密持久化、Mihomo 或文件系统的能力不得由浏览器 fallback 伪造。

#### Scenario: 浏览器打开开发页面

- **WHEN** `window.__TAURI_INTERNALS__` 不存在
- **THEN** 顶栏显示浏览器预览状态
- **AND** 订阅 IPC 操作返回本地运行时要求，而不是模拟成功

### Requirement: 错误可行动且脱敏

错误信息 SHALL 表达错误类别和下一步，不包含秘密原文。单个来源失败不得清空其他已识别来源或用户规则选择。

添加 VLESS URI/Base64 VLESS 来源时 SHALL 在来源卡展示参数级编译预检结果。预检失败 SHALL 清除该来源的格式就绪状态，并区分未知参数、重复参数、别名冲突、security 选项和 transport 选项；多个失败节点 SHALL 一次显示有上限的脱敏汇总。

#### Scenario: 生成快速失败

- **WHEN** 编译在短时间内返回格式、参数或 bootstrap 错误
- **THEN** 主操作附近显示“生成未完成”和安全错误
- **AND** 健康状态不保留“可进入编译”
- **AND** 辅助技术通过 `role=alert` 获得更新。

#### Scenario: 来源参数预检失败

- **WHEN** 已识别 VLESS 来源中的一个或多个节点未通过参数映射
- **THEN** 来源卡显示“参数预检未通过”
- **AND** 不显示“可转换并编译”
- **AND** 安全警告一次列出节点位置和问题类别，不回显任何分享内容。

### Requirement: 域名节点 bootstrap 输入

UI SHALL 为严格编译提供 `source:node=IPv4` 多行映射入口。入口 SHALL 说明 IPv4 必须由用户通过受保护路径确认；系统不得自动直连解析。格式错误、零序号、重复位置和无效 IPv4 SHALL 在 IPC 前阻止编译，且错误不得回显映射值。

#### Scenario: 来源序号可能变化

- **WHEN** 用户移除一个已添加来源
- **THEN** UI 清空全部位置型 bootstrap 映射
- **AND** 不把旧映射静默应用到新的来源/节点位置。

## Implementation Map

| 路径 | 责任 |
|---|---|
| `src/app/App.tsx` | 一级功能切换、页面装配与单列三步工作流布局 |
| `src/app/useStudioWorkspace.ts` | 页面级状态和用例编排 |
| `src/app/App.css` | 桌面布局和组件样式 |
| `src/features/source-inspection/` | 来源功能切片 |
| `src/features/node-converter/` | 独立 VLESS/WireGuard 转换页面、输入状态和直接可见的敏感结果展示 |
| `src-tauri/src/domain/compile/template.rs` | 去敏内置模版组装与 IP 节点引用替换 |
| `src-tauri/src/domain/compile/templates/clash-single-node.yml` | 用户样本删除原节点和原名称后的模版骨架 |
| `src/features/profile-builder/` | 模式、catalog、镜像和浏览器草案切片 |
| `src/features/profile-preview/` | 生成主操作、折叠编译计划和结果展示切片 |
| `src/shared/api/studio.ts` | Tauri IPC adapter |
| `src/shared/contracts.ts` | UI/IPC contracts |
| `src/shared/ui/BrandMark.tsx` | 品牌组件 |
| `src/data/rule-catalog.json` | 浏览器预览 catalog |

## Verification

```bash
npm test
npm run build
```

交互和视觉验收还需要在 Tauri 窗口中分别验证最小尺寸、长警告、多个来源、错误状态和键盘访问；构建通过不代表这些运行态场景已经验证。
