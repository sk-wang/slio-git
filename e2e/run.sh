#!/bin/bash
# E2E RPA 测试运行器 (v2)
# 用法: ./e2e/run.sh [pytest参数...]
#
# 前提:
#   pip3 install -r e2e/requirements.txt
#   系统设置 > 隐私与安全 > 辅助功能 > 允许终端
#
# 示例:
#   ./e2e/run.sh                          # 跑全部（旧测试 + 场景测试）
#   ./e2e/run.sh tests/                   # 只跑单元测试（不需要 GUI）
#   ./e2e/run.sh scenarios/               # 只跑场景测试
#   ./e2e/run.sh -k "提交"               # 按关键字过滤
#   ./e2e/run.sh -m "smoke"              # 按标记过滤
#   ./e2e/run.sh -v -s                    # 详细输出

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# 清理截图和输出目录
rm -rf "$SCRIPT_DIR/screenshots" "$SCRIPT_DIR/output"
mkdir -p "$SCRIPT_DIR/screenshots" "$SCRIPT_DIR/output" "$SCRIPT_DIR/reference_images"

echo "=== slio-git E2E RPA 测试 (v2) ==="
echo "截图目录: $SCRIPT_DIR/screenshots/"
echo "输出目录: $SCRIPT_DIR/output/"
echo ""

cd "$SCRIPT_DIR"
python3 -m pytest "$@" -v --tb=short
