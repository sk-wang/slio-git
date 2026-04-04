# Quickstart: JetBrains风格Git UI重构

**Feature**: JetBrains风格Git UI重构
**Date**: 2026-03-22
**Branch**: `002-jetbrains-ui-refactor`

## 开发环境设置

### 前置条件

- Rust 1.75+ (建议使用 rustup 安装)
- macOS 11+ / Windows 10+ / Ubuntu 20.04+
- Git 2.30+

### 构建步骤

```bash
# 克隆仓库
git clone https://github.com/your-org/slio-git.git
cd slio-git

# 构建项目
cargo build --package src-ui --release

# 运行应用
./target/release/src-ui
```

### 运行测试

```bash
# 运行所有测试
cargo test --workspace

# 运行 git-core 单元测试
cargo test --package git-core

# 运行 UI 测试
cargo test --package src-ui
```

### 代码格式

```bash
# 格式化代码
cargo fmt

# 运行 clippy
cargo clippy --workspace
```

---

## 项目架构

```
slio-git
├── src/git-core/    # Git 操作库（可独立测试）
│   └── src/
│       ├── lib.rs
│       ├── diff.rs        # 三路差异 + 冲突解决
│       ├── repository.rs  # 仓库操作
│       └── ...
│
└── src-ui/         # Iced UI 层
    └── src/
        ├── main.rs        # 应用入口
        ├── state.rs      # 全局状态
        ├── views/        # 主视图
        └── widgets/      # UI 组件
```

---

## 核心开发工作流

### 1. 添加新的 Git 操作

1. 在 `src/git-core/src/` 中添加函数
2. 在 `src/git-core/src/lib.rs` 中导出
3. 编写单元测试
4. 在 `src-ui/src/state.rs` 中通过 `AppState` 暴露给 UI

### 2. 添加新的 UI 组件

1. 在 `src-ui/src/widgets/` 中创建新组件
2. 在 `src-ui/src/views/` 中组合使用
3. 更新 `state.rs` 中的消息类型

### 3. 调试

```bash
# 启用日志
RUST_LOG=debug ./target/release/src-ui

# 查看日志文件
cat ~/Library/Application\ Support/slio-git/logs/*.log
```

---

## 参考资源

- [Iced 文档](https://docs.rs/iced/latest/iced/)
- [git2-rs 文档](https://docs.rs/git2/latest/git2/)
- [IntelliJ Git 插件源码](~/git/intellij-community/plugins/git4idea/)
