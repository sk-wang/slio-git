#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

APP_NAME="slio-git"
APP_DIR="$ROOT_DIR/dist/${APP_NAME}.app"
DMG_ROOT="$ROOT_DIR/dist/dmg-root"
DMG_PATH="$ROOT_DIR/dist/${APP_NAME}.dmg"
MACOS_DIR="$APP_DIR/Contents/MacOS"
RESOURCES_DIR="$APP_DIR/Contents/Resources"
INFO_PLIST="$APP_DIR/Contents/Info.plist"
PKGINFO="$APP_DIR/Contents/PkgInfo"

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

echo "Building release binary..."
cargo build --release -p src-ui

echo "Preparing app bundle..."
rm -rf "$APP_DIR" "$DMG_ROOT"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR" "$DMG_ROOT"

cp "$ROOT_DIR/target/release/src-ui" "$MACOS_DIR/$APP_NAME"
chmod 755 "$MACOS_DIR/$APP_NAME"

ICON_SRC="$ROOT_DIR/src-ui/assets/AppIcon.icns"
if [[ -f "$ICON_SRC" ]]; then
  cp "$ICON_SRC" "$RESOURCES_DIR/AppIcon.icns"
  ICON_KEY='  <key>CFBundleIconFile</key>
  <string>AppIcon</string>'
else
  ICON_KEY=""
fi

cat > "$INFO_PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>zh_CN</string>
  <key>CFBundleDisplayName</key>
  <string>${APP_NAME}</string>
  <key>CFBundleExecutable</key>
  <string>${APP_NAME}</string>
  <key>CFBundleIdentifier</key>
  <string>com.slio.git</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>${APP_NAME}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
${ICON_KEY}
  <key>LSMinimumSystemVersion</key>
  <string>12.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
EOF

printf 'APPL????' > "$PKGINFO"
plutil -lint "$INFO_PLIST" >/dev/null

echo "Staging DMG contents..."
cp -R "$APP_DIR" "$DMG_ROOT/"
ln -s /Applications "$DMG_ROOT/Applications"

echo "Creating DMG..."
rm -f "$DMG_PATH"
hdiutil create \
  -volname "$APP_NAME" \
  -srcfolder "$DMG_ROOT" \
  -ov \
  -format UDZO \
  "$DMG_PATH" >/dev/null

echo "Done:"
echo "  App: $APP_DIR"
echo "  DMG: $DMG_PATH"
