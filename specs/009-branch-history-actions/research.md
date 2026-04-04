# Research: 分支视图提交动作补齐

## 1. 分支视图与历史视图应共享同一套提交时间线组件

- **Decision**: 将当前 `history_view.rs` 的提交图谱、行选中态和基础列表行为抽成共享组件，同时供分支视图与历史视图使用。
- **Rationale**: 用户要的是“在分支视图里像 PhpStorm 一样对提交直接操作”，而不是再维护一套与历史页面不同的提交 UI。共享组件可以保证图谱渲染、滚动、选中、导航和动作锚点在两处保持一致。
- **Alternatives considered**:
  - 直接在 `branch_popup.rs` 复制现有历史列表逻辑：短期快，但极易造成行为分叉
  - 只在分支视图做简化列表、不显示图谱：会削弱对 PhpStorm/IDEA 目标样式的贴近度

## 2. 提交动作的启用 / 禁用需要有独立的资格判定模型

- **Decision**: 把“当前提交是否允许 checkout、建分支、建标签、摘取、回退、重置、推送到这里、fixup、squash、drop”等规则抽成单独的资格判定模型，而不是在 UI 按按钮逐个拼条件。
- **Rationale**: 该特性的核心难点不是把菜单画出来，而是对当前分支、上游、已发布状态、merge/root commit、进行中的 Git 流程进行一致判断。独立模型能让禁用原因、确认提示和测试覆盖都保持一致。
- **Alternatives considered**:
  - 每个按钮在 `branch_popup.rs` 里自己判断：实现看似简单，但条件会迅速失控
  - 不满足条件时直接执行再报错：交互粗糙，且与 IntelliJ 风格不符

## 3. 高风险提交动作优先使用系统 git 命令封装到 `git-core`

- **Decision**: cherry-pick、revert、reset、format-patch、interactive rewrite 相关动作优先通过系统 `git` 命令在 `git-core` 中封装，并保留结构化返回结果。
- **Rationale**: 当前项目的 branch / rebase / tag / remote 已大量使用系统 git 命令，复杂历史动作通过 CLI 更容易贴近真实 Git 语义，也更容易覆盖 interactive rebase、fixup、squash、drop 这类 libgit2 不擅长的流程。
- **Alternatives considered**:
  - 全部改用 libgit2：在部分动作上可行，但 interactive rewrite 与 patch/export 语义更难对齐
  - 把系统 git 直接从 UI 调用：会破坏 library-first 架构并降低测试性

## 4. “推送到这里”需要收缩为当前分支上游的受限发布动作

- **Decision**: 将“推送到这里”定义为：仅对当前分支的已配置上游生效，且选中的提交必须位于当前分支可解释的提交链上；必要时明确提示是否为非快进或不可执行。
- **Rationale**: 这是最符合当前产品心智的最小闭环：用户在分支视图里看着当前分支的历史，决定只把远端推进到某个已知位置。收缩作用域能让风险提示与禁用原因更容易被用户理解。
- **Alternatives considered**:
  - 允许对任意 remote / branch 执行推送到这里：范围过大，风险提示会非常复杂
  - 完全不做此动作：会直接丢掉截图中最有辨识度的一类能力

## 5. rewrite 动作应被视为“引导式会话”，而不是单次按钮

- **Decision**: reword、fixup、squash、drop、从这里开始整理、撤销最近提交都被建模为带边界和后续状态的 rewrite 会话，并尽量复用现有 `rebase_editor` 的继续 / 跳过 / 中止机制。
- **Rationale**: 这些动作经常会进入中间状态或引发冲突；如果把它们当成“点一下就结束”的按钮，UI 很快会在异常路径上断裂。引导式会话更接近 IntelliJ 的处理方式，也更符合 Constitution 对风险状态站内处理的要求。
- **Alternatives considered**:
  - 每个 rewrite 动作都做成独立弹窗：能用，但状态回收困难
  - 只提供命令入口不承接后续流程：用户仍然会被迫回终端

## 6. 创建补丁应该产出真正可保存的 patch 文件，而不是只显示文本

- **Decision**: “创建补丁”以保存 patch 文件为目标，通过保存位置选择 + `git format-patch` 导出可复用的补丁结果。
- **Rationale**: 用户选择“创建补丁”通常是为了把补丁带出当前应用，而不是单纯在弹窗里阅读。生成可保存文件更接近实际协作使用场景，也更贴近桌面 Git 工具预期。
- **Alternatives considered**:
  - 仅把 patch 内容展示在文本窗中：可读，但不方便复用
  - 只支持复制到剪贴板：对短 diff 尚可，对完整提交补丁不实用
