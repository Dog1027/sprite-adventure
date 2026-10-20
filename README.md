# Sprite Adventure

## 簡介

Roguelike + 收集精靈 + 陣型戰鬥。詳細設計見 [docs/DESIGN.md](docs/DESIGN.md)。

## 環境需求

## 建置與執行

### 原生版

### 網頁版（WebAssembly）

## 操作方式

## 專案結構

程式分成精靈（`src/sprite/`）、戰鬥（`src/battle/`）、流程（`src/run/`）三個模組，詳見 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)。

## 協作流程

- 開工前先 `git fetch`，完成一個功能就 commit 並 push
- 各模組由不同人負責；修改模組之間的介面前先討論（見 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)）
- 提交前執行 `cargo test` 與 `cargo clippy`

## 已知問題
