## ADDED Requirements

### Requirement: 提交工作流场景测试
系统 SHALL 提供完整的提交工作流 E2E 测试：打开提交对话框 → 输入提交信息 → 确认提交 → 验证提交成功。

#### Scenario: 正常提交文件
- **WHEN** 测试仓库有未暂存的文件变更
- **THEN** 场景 SHALL 打开提交对话框、输入提交信息、点击提交、验证对话框关闭且历史记录更新

### Requirement: 分支切换场景测试
系统 SHALL 提供分支切换 E2E 测试：打开分支弹窗 → 选择/搜索分支 → 确认切换 → 验证当前分支变更。

#### Scenario: 切换到已有本地分支
- **WHEN** 测试仓库存在 `develop` 分支
- **THEN** 场景 SHALL 通过分支弹窗选择 `develop`，验证状态栏显示的当前分支名更新

#### Scenario: 搜索并切换分支
- **WHEN** 在分支弹窗的搜索框输入分支名关键字
- **THEN** 场景 SHALL 验证列表被过滤，选择目标分支后完成切换

### Requirement: Diff 查看场景测试
系统 SHALL 提供 diff 查看 E2E 测试：修改文件 → 在变更列表中选择文件 → 验证 diff editor 显示正确的变更内容。

#### Scenario: 查看单文件 diff
- **WHEN** 测试仓库中某文件有未暂存变更
- **THEN** 场景 SHALL 在变更列表点击该文件，验证 diff editor 显示绿色(新增)/红色(删除)行

### Requirement: Settings 持久化场景测试
系统 SHALL 提供设置持久化 E2E 测试：打开设置 → 修改选项 → 关闭并重启应用 → 验证设置保持。

#### Scenario: 设置修改后重启保持
- **WHEN** 用户在设置面板修改某个开关选项并关闭应用
- **THEN** 重新启动应用后，该设置项 SHALL 保持修改后的状态

### Requirement: Stash 操作场景测试
系统 SHALL 提供 stash 操作 E2E 测试：stash 变更 → 验证工作区干净 → 恢复 stash → 验证变更恢复。

#### Scenario: Stash 并恢复变更
- **WHEN** 工作区有未提交变更
- **THEN** 场景 SHALL 执行 stash、验证变更列表清空、执行 stash pop、验证变更恢复
