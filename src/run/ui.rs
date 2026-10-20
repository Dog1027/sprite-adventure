//! 簡單的介面元件（即時模式：畫出來的同時判斷點擊）

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
    centered_text(label, rect.center(), 24.0, color);
    hover && clicked()
}

pub fn centered_text(text: &str, center: Vec2, size: f32, color: Color) {
    let m = measure_text(text, None, size as u16, 1.0);
    draw_text(
        text,
        center.x - m.width / 2.0,
        center.y + m.offset_y / 2.0,
        size,
        color,
    );
}
