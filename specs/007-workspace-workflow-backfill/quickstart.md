# Quickstart: 工作区工作流补齐与直改回填

## 1. Build & Verification

```bash
cargo check -p src-ui
cargo test --workspace --no-run
cargo test --workspace
```

## 2. Launch the UI

```bash
cargo run -p src-ui
```

## 3. Workspace Continuity Walkthrough

1. 打开两个不同本地仓库。
2. 关闭应用并重新启动。
3. 确认：
   - 上次仓库会自动恢复
   - 左侧导航区出现最近项目
   - 点击最近项目可以切换到对应仓库
4. 删除最近项目中的某个目录后重新启动，确认无效记录会被清理而不会导致应用卡死。

## 4. Remote Workflow Walkthrough

1. 打开一个当前分支已配置 upstream 的仓库。
2. 保持主工作区空闲，等待自动刷新周期过去。
3. 确认当前分支的同步状态会刷新，但打开辅助面板时自动刷新会暂停。
4. 对 SSH remote 执行 `Fetch` / `Push`：
   - 已配置 ssh-agent 时，无需额外手填凭据即可工作
   - 成功后出现 toast 提示
5. 对 HTTPS remote 执行 `Fetch` / `Pull`：
   - 已配置 credential helper 时，优先复用系统凭据
   - 成功后仓库状态同步更新

## 5. Diff / Commit Walkthrough

1. 打开包含 Rust、PHP、TypeScript、YAML 或 Shell 改动的仓库。
2. 在 unified diff 与 split diff 中查看文件，确认代码高亮正常。
3. 打开提交面板：
   - 勾选 / 取消勾选待提交文件
   - 点击文件项切换预览
   - 确认右侧预览能跟随当前选中文件变化
4. 在 amend 模式下重复一次，确认预览与提交说明仍保持可用。

## 6. Conflict Walkthrough

1. 准备一个带冲突块的仓库。
2. 进入冲突列表页，确认能看到：
   - 冲突文件数
   - 冲突块数
   - 需手工处理数量
   - 可直接处理数量
3. 打开某个文件的三栏合并页，确认：
   - ours / result / theirs 都按代码高亮渲染
   - 可切换上一块 / 下一块
   - 可逐块选择 ours / base / theirs
4. 触发 `自动合并`、`接受全部 ours`、`接受全部 theirs`，确认结果会反映到 result 列并能最终完成解决。

## 7. Runtime Upgrade Sanity Check

1. 启动应用并完成以下高频流程：
   - 打开仓库
   - 选择文件查看 diff
   - 打开提交面板并输入中文说明
   - 打开远程菜单执行一次成功操作
   - 打开冲突页并切换块
2. 确认升级到 iced 0.14 后，上述流程无明显功能性回退。
