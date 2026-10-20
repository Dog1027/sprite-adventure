#!/usr/bin/env bash
# 建置 WebAssembly 版本並輸出到 dist/
# 用法: ./build-web.sh          建置
#       ./build-web.sh serve    建置後在 http://localhost:8000 啟動本機伺服器
set -euo pipefail
cd "$(dirname "$0")"

cargo build --release --target wasm32-unknown-unknown

rm -rf dist && mkdir -p dist
cp web/index.html web/mq_js_bundle.js dist/
cp -r assets dist/
cp target/wasm32-unknown-unknown/release/sprite.wasm dist/

if command -v wasm-opt >/dev/null; then
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals --enable-multivalue --enable-reference-types dist/sprite.wasm -o dist/sprite.wasm
fi
ls -lh dist/sprite.wasm

if [[ "${1:-}" == "serve" ]]; then
    echo "→ http://localhost:8000"
    python3 -m http.server 8000 --directory dist
fi
