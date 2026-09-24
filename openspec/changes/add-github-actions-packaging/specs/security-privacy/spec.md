# Delta: 安全与隐私

## ADDED Requirements

### Requirement: 公共 CI 与发布边界

公共 GitHub Actions SHALL 使用最小令牌权限并固定第三方 Action 的完整 commit SHA。CI 和 Release 不得把订阅 URL、UUID、密码、token、生成 YAML 或其他 secret-bearing 文件作为日志或构建产物上传。

#### Scenario: 普通 CI 运行

- **GIVEN** 仓库包含 secret-bearing 运行时功能
- **WHEN** push 或 pull request 触发 CI
- **THEN** 工作流只使用仓库内的占位测试数据
- **AND** `GITHUB_TOKEN` 只有 `contents: read`。
