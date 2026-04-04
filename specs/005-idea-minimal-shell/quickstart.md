# Quickstart: IDEA 风格的极简 Git 工作台

## 1. Build & Verification

在实现完成后执行：

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test
```

## 2. Launch the UI

```bash
cargo run -p src-ui
```

确认：

- 中文字体正常显示
- 仓库打开后的工作区不再显示常驻产品标题 `slio-git`
- 顶部上下文收敛为单一仓库/分支入口

## 3. Manual Walkthrough

1. 启动应用且不打开仓库，确认欢迎态仍然保留清晰主入口。
2. 打开一个仓库，确认主工作区顶部不再出现多层标题、tagline、chip rows 和重复说明块。
3. 在主工作区识别当前仓库与当前分支，确认它们通过一个主上下文入口呈现。
4. 点击当前分支/上下文入口，确认弹出分支与动作面板。
5. 在面板中验证：
   - 常用动作优先显示
   - 分支列表存在搜索或快速定位能力
   - 最近 / 本地 / 远程分组清晰
6. 回到主工作区，确认改动树与 diff 重新成为主要视觉区域。
7. 执行 `刷新`、`暂存`、`取消暂存`、`提交`，确认反馈更短、更克制，不长期占位。
8. 验证历史、标签、储藏、远程、rebase、冲突处理等能力仍可从新入口层级到达。

## 4. Regression Matrix

| Flow | Steps | Expected Result |
|------|-------|-----------------|
| Repository open | 打开仓库 | 首屏只保留一个主上下文入口，不再显示产品名标题 |
| Context identification | 观察顶部 | 3 秒内可识别仓库与分支 |
| Branch switcher | 点击当前分支 | 弹出集中式分支/动作面板 |
| Changes focus | 返回主区 | 改动树与 diff 占主导面积 |
| Minimal feedback | 执行刷新/暂存 | 反馈简短、明确、非长期占位 |
| Capability reachability | 打开历史/远程/标签/储藏/Rebase | 能力仍可到达且路径清晰 |

## 5. IDEA Comparison Notes

人工对照 `~/git/intellij-community` 时，重点不是像素复刻，而是验证以下原则是否成立：

- 主工作区聚焦内容，而不是品牌/说明
- 分支与次要动作通过弹层渐进展开
- 最近、本地、远程分组明确
- 高频动作层级靠前，低频动作不长期占位
