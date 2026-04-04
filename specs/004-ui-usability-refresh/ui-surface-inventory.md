# UI Surface Inventory

## Scope Snapshot

| Surface | Current Entry | Primary Goal | Redesign Status | Notes |
|---------|---------------|--------------|-----------------|-------|
| Welcome shell | App launch, no repository | 让用户 30 秒内找到主入口 | In Progress | 新欢迎态、主操作卡片、Darcula 壳层已接入 |
| Workspace overview | Open repository | 汇总仓库、分支、变更和下一步 | In Progress | 新概览卡片、侧边导航与状态栏已接入 |
| Change list + diff | Open repository, click file | 选择文件并查看差异 | In Progress | 新变更列表、右侧差异区、上下文件导航已接入 |
| Conflict workflow | Merge/rebase conflict | 优先暴露冲突并进入处理 | In Progress | 新导航入口与冲突列表已接入；写回动作待补 |
| Commit dialog | Toolbar commit | 输入提交消息并提交 | Planned | 当前入口已保留，但对话框尚未接入新壳层 |
| Branch popup | Secondary flow | 管理分支、切换与合并 | Planned | 需要接到统一导航和反馈模型 |
| Stash panel | Secondary flow | 保存/应用/删除 stash | Planned | 入口保留，视图待整合 |
| History view | Secondary flow | 浏览提交历史和详情 | Planned | 视图已存在，待接壳层 |
| Remote dialog | Secondary flow | fetch/pull/push | Planned | 需要接入统一远程反馈 |
| Tag dialog | Secondary flow | 查看/创建/删除 tag | Planned | 需要接入新导航和表单风格 |
| Rebase editor | Exceptional flow | 查看变基状态并继续 | Planned | 需要接入错误/进度反馈 |

## This Implementation Pass

- Phase 1 文档基线已建立。
- Phase 2 壳层状态、反馈模型、Darcula 令牌和共享样式已落地。
- Phase 3 MVP 先覆盖 Welcome、Overview、Changes、Conflicts 四个主界面切面。
- 次级视图在本轮仍保留为下一阶段整合项，已记录到缺陷/后续台账。
