# Change: 公开仓库并增加自动打包

## 状态

`accepted`

## 现状证据与问题

- GitHub 仓库当前为私有仓库，开源用户无法浏览源码或下载公开构建产物。
- 仓库没有 `.github/workflows/`，提交缺少远程验证，版本标签也不会生成桌面安装包。
- 本地已有 npm、Rust、OpenSpec 与架构验证入口，但尚未组合为最小权限的 CI/Release 流程。

## 用户可观察行为

- `tao1587/mihomo-studio` 改为公开仓库。
- 推送与 pull request 自动运行规范、前端和 Rust 检查。
- 推送 `v*` 标签自动为 macOS、Linux 和 Windows 构建安装包，并发布 GitHub Release。

## 影响能力与实现文件

- 主能力：`spec-governance`。
- 安全边界：`security-privacy`。
- 新增 `.github/workflows/ci.yml`、`.github/workflows/release.yml` 与 `scripts/check-github-workflows.mjs`。
- 更新 `package.json`、当前能力 Spec 与 `openspec/spec-index.json` 的 Implementation Map。

## 安全、隐私、兼容与回滚

- CI 默认 `contents: read`；Release 仅授予 `contents: write`。
- Actions 依赖固定到完整 commit SHA，减少可变标签带来的供应链风险。
- 工作流不得输出或上传订阅 URL、UUID、密码、token、生成 YAML 或其他 secret-bearing artifact。
- 桌面包暂不做平台签名；macOS Gatekeeper 与 Windows SmartScreen 可能提示未签名应用。
- 回滚可删除两个 workflow，并把仓库可见性改回私有；已经公开过的提交不能视为从未公开。

## 验收场景

### 提交验证

- **GIVEN** main 分支或 pull request 有新提交
- **WHEN** CI 运行
- **THEN** OpenSpec、前端测试/构建以及 Rust 格式/测试/检查分别执行
- **AND** 工作流令牌只有只读仓库内容权限。

### 标签发布

- **GIVEN** 已同步版本号并创建 `v*` 标签
- **WHEN** Release 工作流运行
- **THEN** 三个平台分别构建原生安装包
- **AND** 安装包上传到该标签对应的公开 GitHub Release。

### 非发布提交

- **GIVEN** 普通分支提交没有 `v*` 标签
- **WHEN** GitHub Actions 处理该提交
- **THEN** 不创建 Release。

## 证据分层

- 源码/配置：workflow 静态检查与 YAML 解析。
- 项目验证：`npm run check:spec`、`npm test`、`npm run build`、Rust 门禁。
- CI：推送后查看 GitHub Actions 结论。
- 安装包与发布：首次推送版本标签后验证；本次不自动创建版本标签。
- Mihomo、文件导出、Clash Verge、规则命中与最终出口：本变更不涉及，未验证。
