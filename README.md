# Sprite Adventure

## 簡介

Roguelike + 收集精靈 + 陣型戰鬥。詳細設計見 [docs/DESIGN.md](docs/DESIGN.md)。

## 環境需求

## 建置與執行

常用指令都寫在 `justfile`，需要先安裝 [just](https://github.com/casey/just)。執行 `just` 會列出全部指令。

| 指令 | 用途 |
|---|---|
| `just run` | 執行原生版 |
| `just test` | 執行單元測試 |
| `just check` | 提交前執行：排版、clippy、測試 |
| `just web` | 建置網頁版到 `dist/` |
| `just serve` | 建置網頁版並在 http://localhost:8000 開啟（同一個網路的手機也能連） |
| `just shots` | 把每個畫面存成 PNG 到 `shots/`，檢查排版用 |

## 操作方式

主要平台是手機（直式），全程點擊操作；電腦上用滑鼠點。

1. **地圖**：由下往上走，點選有黃框的節點前進，最上面是頭目
2. **擺位**：我方在左下、敵方在右上。上方切換陣型；把下方隊伍中的精靈拖到綠色格子（也可以點精靈再點格子）；格子裡的精靈可以拖去別格交換位置，拖到棋盤外就拿下來。黃線標示第一個攻擊目標，金色字是這一格的陣型加成。至少擺 1 隻才能開戰
3. **戰鬥**：自動進行，可以快轉或跳過。打倒頭目即過關，戰敗則重新開始
4. **捕捉**：開局只有 1 隻精靈和 3 個捕捉球；在捕捉節點用一個球選一隻加入隊伍（目前必定成功）。隊伍比較多時用 `<` `>` 翻頁

打贏戰鬥得到金幣，有機會掉捕捉球；商店可以用金幣買捕捉球。休息節點可以選一隻精靈強化 HP、攻擊或防禦。事件尚未實作。

## 專案結構

程式分成精靈（`src/sprite/`）、戰鬥（`src/battle/`）、流程（`src/run/`）三個模組，詳見 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)。

## 協作流程

- 開工前先 `git fetch`，完成一個功能就 commit 並 push
- 各模組由不同人負責；修改模組之間的介面前先討論（見 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)）
- 提交前執行 `just check`（排版、clippy、測試）

## 已知問題

- 原生版要在專案根目錄執行，否則找不到 `assets/` 的字型與精靈圖，中文與角色會無法顯示（用 `just run` 就不會有這個問題）

## 授權與素材

- 字型：[俐方體11號（Cubic 11）](https://github.com/ACh-K/Cubic-11)，SIL Open Font License 1.1，授權文件見 `assets/fonts/OFL.txt`
- 精靈圖：`assets/sprites/` 目前是 `tools/gen_placeholder_sprites.py` 產生的原創占位圖，之後換成正式美術
