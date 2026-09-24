# Delta: Spec 治理

## ADDED Requirements

### Requirement: GitHub 自动化门禁

公开仓库 SHALL 使用最小权限 GitHub Actions：提交与 pull request 运行规范、前端和 Rust 验证；只有 `v*` 标签工作流可获得 `contents: write` 并创建 Release。

#### Scenario: pull request 触发验证

- **GIVEN** 外部贡献者提交 pull request
- **WHEN** GitHub Actions 运行
- **THEN** CI 只读取仓库内容并运行项目门禁
- **AND** 不创建 Release 或上传 secret-bearing artifact。

#### Scenario: 版本标签触发打包

- **GIVEN** 维护者推送 `v*` 标签
- **WHEN** Release 工作流运行
- **THEN** macOS、Linux 和 Windows 安装包被构建
- **AND** 产物发布到该标签的 GitHub Release。
