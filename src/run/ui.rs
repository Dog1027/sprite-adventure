//! 簡單的介面元件（即時模式：畫出來的同時判斷點擊）。座標都是虛擬畫面座標（見 `screen`）。
//! 主要平台是手機，所以不依賴滑鼠移上去（hover）的效果，按鈕至少 44 高方便手指點。

use super::Run;
use crate::gfx;
use crate::palette;
use crate::screen::{H, W};
use crate::sprite::skill::Range;
use crate::sprite::species::Rarity;
use crate::text;
use macroquad::prelude::*;

/// 畫面左右留白
pub const MARGIN: f32 = 16.0;
/// 標題列高度
pub const HEADER_H: f32 = 64.0;
/// 畫面最下方主要按鈕的位置
pub const BOTTOM_BUTTON: Rect = Rect {
    x: MARGIN,
    y: H - 72.0,
    w: W - 2.0 * MARGIN,
    h: 56.0,
};
const RADIUS: f32 = 10.0;

/// 滑鼠或手指的位置（虛擬畫面座標）
pub fn mouse() -> Vec2 {
    crate::screen::to_canvas(mouse_position().into())
}

/// 這一幀是否按下（手機上是手指碰到螢幕）
pub fn clicked() -> bool {
    is_mouse_button_pressed(MouseButton::Left)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// 一般按鈕
    Normal,
    /// 主要動作，一個畫面最多一個（開戰、捕捉、繼續）
    Primary,
    /// 選中的狀態（目前的陣型、選中的精靈）
    Selected,
}

/// 畫一個按鈕，被點擊時回傳 true
pub fn button(rect: Rect, label: &str, style: Style, enabled: bool) -> bool {
    let inside = rect.contains(mouse());
    let held = enabled && inside && is_mouse_button_down(MouseButton::Left);
    let (fill, text_color) = match (enabled, style) {
        (false, _) => (palette::BUTTON_DISABLED, palette::TEXT_MUTED),
        (true, Style::Primary) => (palette::BUTTON_PRIMARY, palette::TEXT_ON_LIGHT),
        (true, Style::Selected) => (palette::BUTTON_ACTIVE, palette::TEXT),
        (true, Style::Normal) => (palette::BUTTON, palette::TEXT),
    };
    // 按住時往下沉一點、變暗
    let r = if held {
        Rect::new(rect.x, rect.y + 2.0, rect.w, rect.h)
    } else {
        rect
    };
    if enabled && !held {
        gfx::shadow(r, RADIUS);
    }
    let fill = if held { gfx::shade(fill, -0.15) } else { fill };
    gfx::rounded_rect_gradient(r, RADIUS, gfx::shade(fill, 0.08), fill);
    gfx::rounded_rect_lines(r, RADIUS, 1.0, palette::BUTTON_BORDER);
    let size = if rect.h >= 50.0 { 24.0 } else { 18.0 };
    text::centered(label, r.center(), size, text_color);
    enabled && inside && clicked()
}

/// 卡片：有陰影的圓角面板
pub fn card(rect: Rect) {
    gfx::shadow(rect, RADIUS);
    gfx::rounded_rect(rect, RADIUS, palette::PANEL);
    gfx::rounded_rect_lines(rect, RADIUS, 1.0, palette::PANEL_BORDER);
}

/// 小標籤，`x` 是左緣、`y` 是中心；回傳寬度
pub fn chip(label: &str, x: f32, y: f32, color: Color) -> f32 {
    let w = text::measure(label, 18.0).width + 16.0;
    let r = Rect::new(x, y - 13.0, w, 26.0);
    gfx::rounded_rect(r, 13.0, palette::CHIP);
    gfx::rounded_rect_lines(r, 13.0, 1.0, gfx::with_alpha(color, 0.5));
    text::centered(label, r.center(), 18.0, color);
    w
}

/// 背景漸層
pub fn background() {
    gfx::gradient(
        Rect::new(0.0, 0.0, W, H),
        palette::BACKGROUND_TOP,
        palette::BACKGROUND_BOTTOM,
    );
}

/// 最上方的標題列；有 `run` 時右邊顯示金幣、捕捉球、隊伍數量
pub fn header(title: &str, run: Option<&Run>) {
    draw_rectangle(0.0, 0.0, W, HEADER_H, palette::HEADER);
    text::draw(title, MARGIN, 46.0, 36.0, palette::TEXT);
    if let Some(run) = run {
        draw_status(run);
    }
}

/// 標題列右側的狀態：金幣、捕捉球、隊伍
fn draw_status(run: &Run) {
    let items: [(Icon, String); 3] = [
        (coin_icon, run.gold.to_string()),
        (ball_icon, run.balls.to_string()),
        (party_icon, run.party.len().to_string()),
    ];
    let mut right = W - MARGIN;
    for (icon, value) in items.iter().rev() {
        let w = text::measure(value, 24.0).width;
        let chip_w = w + 46.0;
        let r = Rect::new(right - chip_w, 16.0, chip_w, 32.0);
        gfx::rounded_rect(r, 16.0, palette::CHIP);
        icon(vec2(r.x + 18.0, r.center().y), 9.0);
        text::draw(value, r.x + 34.0, r.center().y + 9.0, 24.0, palette::TEXT);
        right -= chip_w + 6.0;
    }
}

/// 圖示的畫法：中心點與半徑
pub type Icon = fn(Vec2, f32);

/// 金幣圖示
pub fn coin_icon(center: Vec2, r: f32) {
    draw_circle(center.x, center.y, r, gfx::shade(palette::COIN, -0.3));
    draw_circle(center.x, center.y, r * 0.75, palette::COIN);
}

/// 捕捉球圖示：發光的水晶球，中間一圈金環
pub fn ball_icon(center: Vec2, r: f32) {
    draw_circle(center.x, center.y, r, gfx::shade(palette::ORB, -0.45));
    draw_circle(center.x, center.y - r * 0.08, r * 0.85, palette::ORB);
    draw_circle(
        center.x,
        center.y - r * 0.2,
        r * 0.55,
        gfx::shade(palette::ORB, 0.35),
    );
    draw_line(
        center.x - r * 0.95,
        center.y + r * 0.15,
        center.x + r * 0.95,
        center.y + r * 0.15,
        (r * 0.22).max(1.5),
        palette::ORB_RING,
    );
    // 左上角的反光
    draw_circle(
        center.x - r * 0.35,
        center.y - r * 0.4,
        r * 0.2,
        gfx::with_alpha(palette::TEXT, 0.9),
    );
}

/// 隊伍圖示：一隻小精靈的輪廓
pub fn party_icon(center: Vec2, r: f32) {
    draw_circle(center.x, center.y + r * 0.2, r * 0.8, palette::ALLY);
    draw_triangle(
        center + vec2(-r * 0.75, -r * 0.1),
        center + vec2(-r * 0.35, -r * 0.4),
        center + vec2(-r * 0.6, -r),
        palette::ALLY,
    );
    draw_triangle(
        center + vec2(r * 0.75, -r * 0.1),
        center + vec2(r * 0.35, -r * 0.4),
        center + vec2(r * 0.6, -r),
        palette::ALLY,
    );
}

/// 翻頁的左右箭頭，回傳新的頁數
pub fn pager(left: Rect, right: Rect, page: usize, pages: usize) -> usize {
    let mut page = page.min(pages.saturating_sub(1));
    if button(left, "<", Style::Normal, page > 0) {
        page -= 1;
    }
    if button(right, ">", Style::Normal, page + 1 < pages) {
        page += 1;
    }
    page
}

pub fn range_label(range: Range) -> &'static str {
    match range {
        Range::Single => "單體",
        Range::Row => "貫穿",
        Range::Front => "前排",
        Range::Snipe => "狙擊",
        Range::All => "全體",
        Range::Heal => "治療",
    }
}

pub fn rarity_label(rarity: Rarity) -> (&'static str, Color) {
    match rarity {
        Rarity::Common => ("普通", palette::RARITY_COMMON),
        Rarity::Uncommon => ("少見", palette::RARITY_UNCOMMON),
        Rarity::Rare => ("稀有", palette::RARITY_RARE),
    }
}
