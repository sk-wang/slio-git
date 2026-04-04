# Quickstart: IDEA Git 视图重构

**Branch**: `012-idea-ui-refactor` | **Date**: 2026-04-04

## Prerequisites

- 011-idea-git-parity 分支已合并（91 tests passing）
- Rust toolchain (edition 2021+, stable)
- macOS 11+ / Linux / Windows 10+

## Build & Run

```bash
cd /Users/wanghao/git/slio-git
git checkout 012-idea-ui-refactor
cargo build
cargo run
```

## Test

```bash
cargo test                           # All tests
cargo test -p git-core               # git-core only
cargo test -p src-ui                 # UI only
cargo clippy                         # Lint
```

## Verification Checklist

### 提交历史右键菜单
1. 打开仓库 → 切换到日志标签页
2. 右键任意提交 → 验证菜单包含全部 12 个操作项（按 IDEA 分组）
3. 选中 HEAD → 点击"撤销提交" → 验证改动返回暂存区
4. 选中非 HEAD 提交 → 点击"修改消息" → 验证消息编辑框弹出

### 分支右键菜单
1. 点击分支按钮 → 右键本地分支 → 验证 10 个操作项
2. 右键远程分支 → 验证 5 个操作项
3. 右键当前分支 → 验证"检出""删除"为灰色

### 无 Diff 文件预览
1. 创建新文件 → 在变更列表选中 → 验证右侧显示完整内容（绿色行）
2. 选中二进制文件 → 验证显示"二进制文件"提示

### 标签对话框
1. 打开标签 → 验证 5 个输入组件
2. 输入提交引用 → 点击验证 → 验证结果显示

### 贮藏面板
1. 有改动时点击贮藏 → 验证对话框包含消息编辑器+Keep Index
2. 右键贮藏 → 验证 5 个操作项

### 变基编辑器
1. 发起交互式变基 → 验证工具栏+三列表格+详情面板
2. 拖拽行 → 验证顺序更新
3. 双击消息 → 验证行内编辑
