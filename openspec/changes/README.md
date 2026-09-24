# OpenSpec 变更工件

行为变化在实现前创建 `openspec/changes/<change-id>/`。纯文字修正且不改变行为时可以直接同步当前 Spec。

## 最小结构

```text
openspec/changes/<change-id>/
├── proposal.md   # 问题、目标、非目标、影响能力和风险
├── tasks.md      # 可验证的实施步骤
└── specs/
    └── <capability>/spec.md  # 只写新增/修改/删除的 requirement delta
```

## Proposal 必填项

- 现状证据与问题；
- 用户可观察行为；
- 受影响能力和实现文件；
- 安全、隐私、兼容和回滚影响；
- 验收场景；
- 源码、配置、Mihomo、导出、外部加载、运行态分别需要什么证据。

## 生命周期

1. `proposed`：只有提案和 delta Spec；
2. `accepted`：范围得到确认，可以实施；
3. `implemented`：代码和测试完成，但尚未合并到当前能力真相；
4. `archived`：delta 已并入 `openspec/specs/`，工件仅保留决策历史。

不得用 roadmap checkbox 代替行为契约，也不得在实现完成后只更新 change 而不更新当前能力 Spec。
