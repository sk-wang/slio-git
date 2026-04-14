#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TARGET="${WINDOWS_TARGET:-x86_64-pc-windows-gnu}"
APP_NAME="slio-git"
ARCH="${WINDOWS_ARCH:-$(printf '%s' "$TARGET" | cut -d- -f1)}"
DIST_DIR="$ROOT_DIR/dist"
STAGING_DIR="$DIST_DIR/windows-root"
PACKAGE_BASENAME="${APP_NAME}-windows-${ARCH}"
PACKAGE_DIR="$STAGING_DIR/${PACKAGE_BASENAME}"
ZIP_PATH="$DIST_DIR/${PACKAGE_BASENAME}.zip"

VERSION="$(
python3 - <<'PY'
from pathlib import Path
import re

text = Path("Cargo.toml").read_text()
match = re.search(
    r"\[workspace\.package\](?:.*?\n)*?version\s*=\s*\"([^\"]+)\"",
    text,
    re.S,
)
if not match:
    raise SystemExit("Failed to read workspace version from Cargo.toml")
print(match.group(1))
PY
)"

if ! rustup target list --installed | grep -qx "$TARGET"; then
  echo "Installing Rust target: $TARGET"
  rustup target add "$TARGET"
fi

echo "Building Windows release binary for $TARGET..."
cargo build --locked --release -p src-ui --target "$TARGET"

echo "Preparing ZIP contents..."
rm -rf "$STAGING_DIR"
mkdir -p "$PACKAGE_DIR"

cp "$ROOT_DIR/target/$TARGET/release/src-ui.exe" "$PACKAGE_DIR/${APP_NAME}.exe"
cat > "$PACKAGE_DIR/README.txt" <<EOF
slio-git ${VERSION}
=================

1. Extract this archive.
2. Run ${APP_NAME}.exe.

Source:
https://github.com/sk-wang/slio-git
EOF

echo "Creating ZIP archive..."
rm -f "$ZIP_PATH"
python3 - <<PY
from pathlib import Path
import zipfile

root = Path(r"$STAGING_DIR")
package = Path(r"$PACKAGE_DIR")
zip_path = Path(r"$ZIP_PATH")

with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
    for path in package.rglob("*"):
        zf.write(path, path.relative_to(root))
PY

echo "Done:"
echo "  ZIP: $ZIP_PATH"
