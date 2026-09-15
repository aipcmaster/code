#!/usr/bin/env bash
# 把 SPM 构建出的可执行文件打包成最小 .app（无需 Xcode 工程）。
# 用法：bash scripts/make-app.sh （在 macOS 上执行）
set -euo pipefail
cd "$(dirname "$0")/.."

echo "▶ swift build -c release（product AIPCMaster）…"
swift build -c release --product AIPCMaster

APP=build/AIPCMaster.app
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"

BIN_DIR=$(swift build -c release --show-bin-path)
cp "$BIN_DIR/AIPCMaster" "$APP/Contents/MacOS/AIPCMaster"

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>AIPCMaster</string>
    <key>CFBundleIdentifier</key>
    <string>com.aipcmaster.desktop</string>
    <key>CFBundleName</key>
    <string>AIPCMaster</string>
    <key>CFBundleDisplayName</key>
    <string>AIPCMaster（AI电脑大师）</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>13.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.utilities</string>
</dict>
</plist>
PLIST

echo "✅ build/AIPCMaster.app 已生成"
echo "   运行：open build/AIPCMaster.app"
echo "   提示：本机构建未签名，首次打开请在 系统设置→隐私与安全性 中允许，或右键→打开"