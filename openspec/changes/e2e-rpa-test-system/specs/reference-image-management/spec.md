## ADDED Requirements

### Requirement: 参考图按功能分组存储
参考截图 SHALL 存储在 `e2e/refs/<category>/` 目录下，按 UI 功能区域分组（如 `toolbar/`、`branch_popup/`、`commit_dialog/`、`diff_editor/`）。

#### Scenario: 新增参考图
- **WHEN** 开发者需要添加工具栏刷新按钮的参考图
- **THEN** 开发者 SHALL 将截图保存为 `e2e/refs/toolbar/refresh_btn.png`，图片仅包含目标元素及最小必要边距

#### Scenario: 参考图命名规范
- **WHEN** 创建新参考图文件
- **THEN** 文件名 SHALL 使用 snake_case，描述元素功能（如 `commit_dialog_title.png`），禁止使用坐标或日期作为文件名

### Requirement: 模糊匹配与置信度阈值
图像匹配 SHALL 使用可配置的 confidence 阈值（默认 0.8），支持灰度模式匹配以提高跨主题容忍度。

#### Scenario: 默认阈值匹配
- **WHEN** 调用图像匹配且未指定 confidence 参数
- **THEN** 系统 SHALL 使用 0.8 作为默认匹配阈值

#### Scenario: 灰度模式匹配
- **WHEN** 调用 `driver.find_image("btn.png", grayscale=True)`
- **THEN** 系统 SHALL 将屏幕截图和参考图都转为灰度后进行匹配，提高对颜色微调的容忍度

### Requirement: 截图断言对比
系统 SHALL 支持将实际截图与参考图进行像素级对比，输出差异百分比和差异高亮图。

#### Scenario: 截图断言通过
- **WHEN** 实际截图与参考图差异低于阈值（默认 5%）
- **THEN** 断言 SHALL 通过

#### Scenario: 截图断言失败时输出差异图
- **WHEN** 实际截图与参考图差异超过阈值
- **THEN** 系统 SHALL 生成差异高亮图保存到 `e2e/output/diff_<name>.png`，并在测试报告中显示差异百分比
