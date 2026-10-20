//! 配色表：所有畫面的顏色都從這裡取，換美術時只要改這個檔案。
//! 依「用途」命名，不依顏色命名（例如 `ALLY` 而不是 `SKYBLUE`），同樣用途的地方會一起變。

use macroquad::prelude::Color;

/// 以 0x RRGGBB 指定顏色
const fn hex(rgb: u32) -> Color {
    rgba(rgb, 1.0)
}

/// 以 0x RRGGBB 與透明度指定顏色
const fn rgba(rgb: u32, a: f32) -> Color {
    Color::new(
        ((rgb >> 16) & 0xff) as f32 / 255.0,
        ((rgb >> 8) & 0xff) as f32 / 255.0,
        (rgb & 0xff) as f32 / 255.0,
        a,
    )
}

// 背景與面板
/// 背景由上到下的漸層
pub const BACKGROUND_TOP: Color = hex(0x1f2238);
pub const BACKGROUND_BOTTOM: Color = hex(0x111220);
/// 畫面兩側的黑邊、換畫面時的淡入
pub const LETTERBOX: Color = hex(0x08080d);
/// 最上方標題列
pub const HEADER: Color = rgba(0x000000, 0.25);
/// 卡片、面板
pub const PANEL: Color = hex(0x272b44);
pub const PANEL_BORDER: Color = rgba(0xffffff, 0.08);
/// 面板裡再凸起一層（例如標籤）
pub const CHIP: Color = rgba(0x000000, 0.3);
/// 小標籤（技能名稱、治療量）的底色
pub const TAG_BG: Color = rgba(0x0d0e16, 0.9);

// 文字
/// 標題、主要文字
pub const TEXT: Color = hex(0xf4f1e8);
/// 說明文字
pub const TEXT_DIM: Color = hex(0xb3b7cc);
/// 不重要或停用的文字
pub const TEXT_MUTED: Color = hex(0x6c7190);
/// 畫在亮色底上的文字（主要按鈕、地圖節點）
pub const TEXT_ON_LIGHT: Color = hex(0x1b1d2a);
/// 金幣、捕捉率、陣型加成等「好處」
pub const HIGHLIGHT: Color = hex(0xf6c453);
/// 提醒（金幣不夠、沒有捕捉球）
pub const WARNING: Color = hex(0xff9f43);
/// 勝利、過關、強化成功
pub const SUCCESS: Color = hex(0x6fdc8c);
/// 戰敗、冒險失敗
pub const FAILURE: Color = hex(0xff5d6c);

// 按鈕
pub const BUTTON: Color = hex(0x363c5e);
/// 選中的按鈕（目前的陣型、選中的精靈）
pub const BUTTON_ACTIVE: Color = hex(0x4a7dff);
/// 主要動作（開戰、捕捉、繼續）
pub const BUTTON_PRIMARY: Color = hex(0xff8a3d);
pub const BUTTON_DISABLED: Color = hex(0x252840);
pub const BUTTON_BORDER: Color = rgba(0xffffff, 0.1);

// 棋盤與精靈
pub const GRID: Color = rgba(0xffffff, 0.07);
pub const BOARD_ALLY: Color = rgba(0x4a7dff, 0.08);
pub const BOARD_ENEMY: Color = rgba(0xff5d6c, 0.08);
/// 擺位畫面中陣型開放的格子
pub const OPEN_SLOT: Color = rgba(0x6fdc8c, 0.16);
pub const OPEN_SLOT_BORDER: Color = rgba(0x6fdc8c, 0.5);
pub const ALLY: Color = hex(0x5aa9ff);
pub const ENEMY: Color = hex(0xff6b81);
pub const DEFEATED: Color = hex(0x454860);
pub const HP_BAR: Color = hex(0x6fdc8c);
/// 血量低於三成
pub const HP_BAR_LOW: Color = hex(0xff5d6c);
pub const HP_BAR_BG: Color = rgba(0x000000, 0.45);

// 戰鬥效果
pub const ATTACK_LINE: Color = hex(0xff9f43);
/// 技能的連線與名稱
pub const SKILL: Color = hex(0xf6c453);
pub const HEAL: Color = hex(0x6fdc8c);
pub const DAMAGE: Color = hex(0xff5d6c);
/// 擺位預覽線（透明度依是否平手另外調整）
pub const PREVIEW_LINE: Color = rgba(0xffd166, 0.9);

// 稀有度
pub const RARITY_COMMON: Color = hex(0xb3b7cc);
pub const RARITY_UNCOMMON: Color = hex(0x6cc4ff);
pub const RARITY_RARE: Color = hex(0xf6c453);

// 圖示
pub const COIN: Color = hex(0xf6c453);
/// 捕捉球：發光的水晶球（原創造型）
pub const ORB: Color = hex(0x4fd1c5);
pub const ORB_RING: Color = hex(0xf6c453);

// 地圖
pub const NODE_WILD: Color = hex(0x7bd389);
pub const NODE_ELITE: Color = hex(0xff9f43);
pub const NODE_SHOP: Color = hex(0xf6c453);
pub const NODE_CATCH: Color = hex(0xff8fb1);
pub const NODE_REST: Color = hex(0x6cc4ff);
pub const NODE_EVENT: Color = hex(0xb58cff);
pub const NODE_BOSS: Color = hex(0xff5d6c);
/// 可以走的節點外圈
pub const NODE_RING: Color = hex(0xffd166);
/// 目前所在節點的外圈
pub const NODE_FOCUS: Color = hex(0xf4f1e8);
/// 走過的路線
pub const PATH_TAKEN: Color = rgba(0xf4f1e8, 0.8);
pub const PATH: Color = rgba(0xffffff, 0.14);
