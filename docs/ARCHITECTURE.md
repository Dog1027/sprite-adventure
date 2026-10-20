# 程式架構

程式分成三個模組：**精靈**、**戰鬥**、**流程**，可以分給不同人負責。遊戲規則見 [DESIGN.md](DESIGN.md)。

## 目錄

```
src/
├── main.rs            入口：建立視窗，每幀呼叫 run::Game
├── rng.rs             共用亂數（同種子可重現）
├── sprite/            精靈
│   ├── mod.rs         Stats（能力值＋節奏數值）、Sprite
│   └── species.rs     讀取精靈種類資料（data/species.toml）
├── battle/            戰鬥
│   ├── mod.rs         棋盤、Unit、Battle（回合進行與結算）、Event、Outcome
│   ├── damage.rs      傷害公式（屬性倍率之後也加在這裡）
│   ├── formation.rs   陣型：開放格子、陣型加成、把精靈擺上棋盤
│   ├── targeting.rs   普攻目標選擇、擺位預覽
│   └── view.rs        戰鬥畫面，棋盤與精靈的畫法（擺位畫面共用）
└── run/               流程
    ├── mod.rs         Run：一局的狀態（隊伍、陣型、地圖位置）、產生敵方隊伍
    ├── map.rs         分支地圖產生
    ├── game.rs        Game：畫面切換（地圖 → 擺位 → 戰鬥 → 結果）
    ├── map_view.rs    地圖畫面
    ├── placement.rs   陣型選擇與擺位畫面
    └── ui.rs          按鈕等簡單介面元件

data/
└── species.toml       精靈種類的能力值與節奏數值（編譯時嵌入程式）
```

## 依賴方向

```
main ──→ run ──→ battle ──→ sprite
          └──────────────────↗
```

- **精靈**不依賴其他遊戲模組
- **戰鬥**只依賴精靈（讀 `Sprite`、`Stats`）
- **流程**依賴戰鬥與精靈
- 不可以反過來依賴，例如戰鬥不能引用流程

## 各模組負責範圍

| 模組 | 負責 | 之後要做 |
|---|---|---|
| 精靈 `sprite/` | 能力值、精靈資料、技能資料、成長、捕捉機率 | 技能定義、成長／進化、捕捉 |
| 戰鬥 `battle/` | 棋盤、回合節奏、目標、傷害、結算、陣型、擺位預覽、戰鬥畫面 | 技能效果執行、治療、各陣型的實際效果 |
| 流程 `run/` | 一局的狀態、分支地圖、各種節點、畫面切換、擺位畫面 | 商店、休息、事件、捕捉流程、陣型解鎖、多層 |

陣型的「定義與加成」屬於戰鬥；「選陣型、擺精靈的畫面」屬於流程，結果以 `Unit`（精靈＋位置）交給戰鬥。

## 模組之間的介面

這些是多人共用的介面，**修改前先跟其他人討論**：

| 介面 | 位置 | 用途 |
|---|---|---|
| `Stats`、`Sprite` | `sprite/mod.rs` | 精靈 → 戰鬥、流程 |
| `species::create` | `sprite/species.rs` | 流程用來建立精靈 |
| `Unit::new(&Sprite, Side, Pos)` | `battle/mod.rs` | 把精靈放到某一格（不含陣型加成） |
| `Formation`、`Formation::place`、`formation::samples` | `battle/formation.rs` | 流程依陣型擺位，取得含陣型加成的 `Unit` |
| `preview`、`view::draw_preview` 等畫法 | `battle/targeting.rs`、`battle/view.rs` | 擺位畫面顯示目標預覽 |
| `Battle::new`、`step`、`outcome` | `battle/mod.rs` | 流程開始戰鬥、取得勝負 |
| `BattleScreen` | `battle/view.rs` | 流程／main 顯示戰鬥 |

一場戰鬥的流程：

1. 流程用 `Formation::place` 把我方與敵方精靈擺進陣型
2. `Battle::new(units, seed)` 建立戰鬥，交給 `BattleScreen` 播放
3. 戰鬥結束後讀 `battle.outcome`（`Win`／`Lose`），流程決定下一步

## 協作規則

- 每個模組的內部可以自由修改；改到上表的介面要先討論
- 規則有變動時先更新 DESIGN.md
- 每個模組的邏輯寫單元測試，提交前執行 `cargo test` 和 `cargo clippy`
- 畫面程式和邏輯分開：邏輯（例如 `battle/mod.rs`、`targeting.rs`、`run/map.rs`）不呼叫 macroquad，才能寫單元測試

## 已決定的介面規則

- ✅ 戰鬥結束後 HP 不保留，每場戰鬥都以滿血開始，所以 `Battle` 只需要回傳勝負
- ✅ 技能的資料（名稱、數值、範圍等定義）放在**精靈**模組；技能效果的執行放在戰鬥模組，因為戰鬥不能被精靈引用
