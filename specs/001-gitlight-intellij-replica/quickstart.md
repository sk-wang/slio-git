# Quickstart: slio-git 开发环境设置

**Date**: 2026-03-22
**Feature**: IntelliJ-Compatible Git Client (Pure Iced)

## 前置要求

### 必需工具

| 工具 | 版本 | 说明 |
|------|------|------|
| Rust | 1.75+ | 使用 rustup 安装 |
| Cargo | 最新 | Rust 包管理器 |
| Git | 2.40+ | 系统 git |

### 可选工具

| 工具 | 版本 | 说明 |
|------|------|------|
| Visual Studio Code | 最新 | IDE 推荐 |
| rust-analyzer | 最新 | Rust 语言支持 |
| CodeLLDB | 最新 | Rust 调试器 |

## 环境安装

### 1. 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustc --version  # 验证安装
```

### 2. 克隆项目

```bash
git clone https://github.com/your-org/slio-git.git
cd slio-git
```

### 3. 安装依赖

```bash
# 构建项目
cargo build

# 运行测试
cargo test

# 运行应用
cargo run --package src-ui
```

## 项目结构

```
slio-git/
├── src/
│   └── git-core/        # Git 核心库 (Library-First)
├── src-ui/              # Pure Iced UI 应用
└── tests/               # 测试
```

## 开发工作流

### 日常开发

```bash
# 1. 确保在正确的分支
git checkout -b my-feature

# 2. 运行测试
cargo test

# 3. 启动应用
cargo run --package src-ui

# 4. 在 src-ui/src/main.rs 修改 UI
#    在 src/git-core/src/lib.rs 修改 Git 逻辑
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定包的测试
cargo test -p git-core

# 运行集成测试
cargo test --test integration

# 运行 parity 测试 (对比 IntelliJ 行为)
cargo test --test parity
```

### 代码检查

```bash
# 格式化代码
cargo fmt

# Lint 检查
cargo clippy -- -D warnings
```

## 调试

### Iced UI 调试

```rust
// 在 src-ui/src/main.rs 中启用日志
env_logger::Builder::from_env(
    env_logger::Env::default().default_filter_or("debug")
).init();
```

### Git 操作调试

```rust
// 启用 git2-rs 调试日志
GIT_TRACE=1 cargo run --package src-ui
```

### 常见问题

| 问题 | 解决方案 |
|------|----------|
| 编译错误: "linked node is not a git repository" | 确保在 git 仓库内运行，或设置 REPO_PATH |
| UI 无响应 | 检查是否在后台线程运行 git 操作 |
| 中文显示乱码 | 确保使用 `Shaping::Advanced` |

## 性能分析

```bash
# 发布构建
cargo build --release

# 性能目标
# - 启动时间: < 300ms
# - 内存占用: < 80MB (空闲)
# - 二进制大小: < 15MB
```

## 下一步

- 阅读 [data-model.md](./data-model.md) 了解数据模型
- 阅读 [plan.md](./plan.md) 了解实施计划
- 运行 `/speckit.tasks` 生成任务列表
