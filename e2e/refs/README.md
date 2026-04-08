# 参考图管理规范

## 目录结构

```
refs/
├── toolbar/          # 工具栏按钮（刷新、提交、设置等）
├── branch_popup/     # 分支弹窗（标题、搜索框、分支项）
├── commit_dialog/    # 提交对话框（标题、输入框、按钮）
├── diff_editor/      # diff 编辑器（新增行、删除行、标题栏）
├── settings/         # 设置面板（标题、checkbox、保存按钮）
└── common/           # 通用元素（状态栏、Tab 标签等）
```

## 命名规范

- 使用 **snake_case**，描述元素功能
- 好: `commit_dialog_title.png`, `refresh_btn.png`
- 坏: `btn1.png`, `2024_01_01_screenshot.png`, `100_200.png`

## 截图要求

1. **最小边距**: 仅包含目标元素及 2-4px 边距
2. **标准窗口**: 在 1728x1080 最大化窗口下截取
3. **PNG 格式**: 无损压缩，保持像素精确
4. **不含鼠标**: 截图时鼠标移开目标区域

## 更新流程

1. UI 变更后，运行 `python3 -c "import driver; driver.region(rx, ry, rw, rh, 'name')"` 截取新参考图
2. 将截图从 `screenshots/` 移至对应的 `refs/<category>/` 目录
3. 运行受影响的测试验证匹配
4. 提交更新的参考图

## 匹配参数

- 默认 confidence: `0.8`
- 灰度模式: 对颜色微调更容忍（如主题切换）
- 区域限定: 对大图搜索指定 region 加速匹配
