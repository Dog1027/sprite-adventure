//! 簡單的介面元件（即時模式：畫出來的同時判斷點擊）

use super::Run;
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
        (false, _, _) => Color::new(0.25, 0.25, 0.28, 1.0),
        (true, true, _) => Color::new(0.2, 0.45, 0.7, 1.0),
        (true, false, true) => Color::new(0.4, 0.4, 0.45, 1.0),
        (true, false, false) => Color::new(0.32, 0.32, 0.36, 1.0),
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, LIGHTGRAY);
    let color = if enabled { WHITE } else { GRAY };
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
    text::draw(&status, 760.0 - w, 50.0, 24.0, WHITE);
}
