//! 簡單的介面元件（即時模式：畫出來的同時判斷點擊）

use super::Run;
use crate::palette;
use crate::text;
use macroquad::prelude::*;

pub fn mouse() -> Vec2 {
    mouse_position().into()
}

pub fn clicked() -> bool {
    is_mouse_button_pressed(MouseButton::Left)
}

/// 畫一個按鈕，被點擊時回傳 true
pub fn button(rect: Rect, label: &str, enabled: bool, active: bool) -> bool {
    let hover = enabled && rect.contains(mouse());
    let bg = match (enabled, active, hover) {
        (false, _, _) => palette::BUTTON_DISABLED,
        (true, true, _) => palette::BUTTON_ACTIVE,
        (true, false, true) => palette::BUTTON_HOVER,
        (true, false, false) => palette::BUTTON,
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, palette::BUTTON_BORDER);
    let color = if enabled {
        palette::TEXT
    } else {
        palette::TEXT_MUTED
    };
    text::centered(label, rect.center(), 24.0, color);
    hover && clicked()
}

/// 右上角的狀態列：金幣、捕捉球、隊伍
pub fn draw_status(run: &Run) {
    let status = format!(
        "金幣 {}　捕捉球 {}　隊伍 {} 隻",
        run.gold,
        run.balls,
        run.party.len()
    );
    let w = text::measure(&status, 24.0).width;
    text::draw(&status, 760.0 - w, 50.0, 24.0, palette::TEXT);
}
