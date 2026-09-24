# Delta: 桌面体验

## ADDED Requirements

### Requirement: 步骤导航跟随滚动

左侧三步导航 SHALL 根据当前页面区段更新 active 序号，并 SHALL 使用 `aria-current="step"` 表达当前步骤。桌面右侧预览栏发生内部滚动时，当前步骤 SHALL 为 03。

#### Scenario: 页面依次滚过三步

- **WHEN** 用户从节点来源滚动到模式与规则，再到预览锚点
- **THEN** active 序号依次为 01、02、03
- **AND** 任一时刻只有一个步骤具有 `aria-current="step"`

### Requirement: 预览主操作可达

桌面三栏布局中，生成主操作 SHALL 固定在预览栏底部，不因预览内容滚动而离开可视区域。窄窗口 SHALL 保持正常文档流，生成按钮位于预览内容末尾。

#### Scenario: 长预览内容

- **GIVEN** 预览结果或警告超过预览栏高度
- **WHEN** 用户滚动预览内容
- **THEN** 生成按钮保持可见且可操作
- **AND** 预览内容不被永久遮挡
