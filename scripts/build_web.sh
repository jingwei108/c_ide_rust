#!/usr/bin/env bash
set -e

# Cide Web 本地构建脚本
# 用法：bash scripts/build_web.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

NATIVE_DIR="$PROJECT_ROOT/native"
FLUTTER_DIR="$PROJECT_ROOT/CideFlutter"

echo "==> 检查 wasm32 target"
rustup target list --installed | grep -q wasm32-unknown-unknown || rustup target add wasm32-unknown-unknown

echo "==> 检查 wasm-pack"
command -v wasm-pack >/dev/null 2>&1 || cargo install wasm-pack

echo "==> 构建 Rust WASM"
cd "$NATIVE_DIR"
wasm-pack build --target web --out-dir "$FLUTTER_DIR/web/pkg" --no-opt

echo "==> 构建 Flutter Web"
cd "$FLUTTER_DIR"
flutter build web --release

echo "==> 添加 SPA 回退规则 (GitHub/Gitee Pages 使用 404.html)"
cp "$FLUTTER_DIR/build/web/index.html" "$FLUTTER_DIR/build/web/404.html"

echo "==> 构建完成"
echo "产物目录: $FLUTTER_DIR/build/web"
echo "本地预览: cd $FLUTTER_DIR/build/web && python -m http.server 8080"
