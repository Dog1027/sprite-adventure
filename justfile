# 常用指令。執行 `just` 列出全部，例如 `just run`、`just check`。
# just 會在專案根目錄執行，所以字型等素材一定找得到。

# 列出所有指令
default:
    @just --list

# 執行原生版
run:
    cargo run

# 執行原生版（最佳化，比較順）
run-release:
    cargo run --release

# 執行單元測試
test:
    cargo test

# 自動排版
fmt:
    cargo fmt

# 檢查排版與 clippy 警告（不修改檔案）
lint:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings

# 提交前執行：排版、clippy、測試
check: lint test

# 建置網頁版到 dist/
web:
    ./build-web.sh

# 建置網頁版並在 http://localhost:8000 開啟本機伺服器
serve:
    ./build-web.sh serve

# 刪除建置產物
clean:
    cargo clean
    rm -rf dist
