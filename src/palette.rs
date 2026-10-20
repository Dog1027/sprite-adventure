//! 配色表：所有畫面的顏色都從這裡取，換美術時只要改這個檔案。
//! 依「用途」命名，不依顏色命名（例如 `ALLY` 而不是 `SKYBLUE`），同樣用途的地方會一起變。

use macroquad::prelude::*;

// 背景與面板
pub const BACKGROUND: Color = Color::new(0.15, 0.15, 0.18, 1.0);
/// 卡片（捕捉、商店）的底色
pub const PANEL: Color = Color::new(0.22, 0.22, 0.26, 1.0);
pub const PANEL_BORDER: Color = GRAY;
/// 小標籤（技能名稱）的底色
pub const TAG_BG: Color = Color::new(0.1, 0.1, 0.12, 0.9);

// 文字
/// 標題、主要文字
pub const TEXT: Color = WHITE;
/// 說明文字、「點擊繼續」
pub const TEXT_DIM: Color = LIGHTGRAY;
/// 不重要或停用的文字
pub const TEXT_MUTED: Color = GRAY;
/// 畫在亮色底上的文字（精靈色塊、地圖節點）
pub const TEXT_ON_LIGHT: Color = BLACK;
/// 金幣、捕捉率、陣型加成等「好處」
pub const HIGHLIGHT: Color = GOLD;
/// 提醒（金幣不夠、沒有捕捉球）
pub const WARNING: Color = ORANGE;
/// 勝利、過關、強化成功
pub const SUCCESS: Color = GREEN;
/// 戰敗、冒險失敗
pub const FAILURE: Color = RED;

// 按鈕
pub const BUTTON: Color = Color::new(0.32, 0.32, 0.36, 1.0);
pub const BUTTON_HOVER: Color = Color::new(0.4, 0.4, 0.45, 1.0);
/// 選中的按鈕（目前的陣型、選中的精靈）
pub const BUTTON_ACTIVE: Color = Color::new(0.2, 0.45, 0.7, 1.0);
pub const BUTTON_DISABLED: Color = Color::new(0.25, 0.25, 0.28, 1.0);
pub const BUTTON_BORDER: Color = LIGHTGRAY;

// 棋盤與精靈
pub const GRID: Color = GRAY;
/// 擺位畫面中陣型開放的格子
pub const OPEN_SLOT: Color = Color::new(0.2, 0.35, 0.25, 1.0);
pub const ALLY: Color = SKYBLUE;
pub const ENEMY: Color = PINK;
pub const DEFEATED: Color = Color::new(0.3, 0.3, 0.3, 1.0);
pub const HP_BAR: Color = GREEN;
pub const HP_BAR_BG: Color = DARKGRAY;

// 戰鬥效果
pub const ATTACK_LINE: Color = ORANGE;
/// 技能的連線與名稱
pub const SKILL: Color = GOLD;
pub const HEAL: Color = GREEN;
pub const DAMAGE: Color = RED;
/// 擺位預覽線（透明度依是否平手另外調整）
pub const PREVIEW_LINE: Color = Color::new(1.0, 0.8, 0.2, 0.9);

// 地圖
pub const NODE_WILD: Color = Color::new(0.5, 0.75, 0.5, 1.0);
pub const NODE_ELITE: Color = ORANGE;
pub const NODE_SHOP: Color = GOLD;
pub const NODE_CATCH: Color = PINK;
pub const NODE_REST: Color = SKYBLUE;
pub const NODE_EVENT: Color = VIOLET;
pub const NODE_BOSS: Color = RED;
/// 可以走的節點外圈
pub const NODE_RING: Color = YELLOW;
/// 滑鼠移上去的節點外圈、目前所在節點
pub const NODE_FOCUS: Color = WHITE;
/// 走過的路線
pub const PATH_TAKEN: Color = WHITE;
pub const PATH: Color = DARKGRAY;
