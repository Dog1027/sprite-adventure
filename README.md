# Sprite Adventure

## 簡介

Roguelike + 收集精靈 + 陣型戰鬥。詳細設計見 [docs/DESIGN.md](docs/DESIGN.md)。

## 環境需求

## 建置與執行

### 原生版

### 網頁版（WebAssembly）

## 操作方式

全程使用滑鼠：

1. **地圖**：點選有黃框的節點前進（野 野生、菁 菁英、商 商店、休 休息、事 事件、王 頭目）
2. **擺位**：上方切換陣型；點隊伍中的精靈，再點綠色格子放進去；點格子裡的精靈可以拿起來。黃線與 `→E2` 標示第一個攻擊目標。至少擺 1 隻才能開戰
3. **戰鬥**：自動進行，結束後點一下繼續。打倒頭目即過關，戰敗則重新開始

商店、休息、事件尚未實作。

## 專案結構

程式分成精靈（`src/sprite/`）、戰鬥（`src/battle/`）、流程（`src/run/`）三個模組，詳見 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)。

## 協作流程

- 開工前先 `git fetch`，完成一個功能就 commit 並 push
- 各模組由不同人負責；修改模組之間的介面前先討論（見 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)）
- 提交前執行 `cargo test` 與 `cargo clippy`

## 已知問題

- 原生版要在專案根目錄執行（`cargo run`），否則找不到 `assets/fonts/` 的字型，中文會無法顯示

## 授權與素材

- 字型：[俐方體11號（Cubic 11）](https://github.com/ACh-K/Cubic-11)，SIL Open Font License 1.1，授權文件見 `assets/fonts/OFL.txt`
