//! 共用的圖形：圓角矩形、漸層、陰影。所有畫面（含戰鬥）都用這裡的畫法，外觀才會一致。
//! 顏色一律從 `palette` 取。

use macroquad::models::{Mesh, Vertex, draw_mesh};
use macroquad::prelude::*;

/// 每個圓角用幾段折線
const CORNER_SEGMENTS: usize = 6;

/// 圓角矩形外框上的點（順時針，從左上角開始）
fn outline(r: Rect, radius: f32) -> Vec<Vec2> {
    let rad = radius.min(r.w / 2.0).min(r.h / 2.0).max(0.0);
    let corners = [
        (vec2(r.x + rad, r.y + rad), 180.0_f32),
        (vec2(r.x + r.w - rad, r.y + rad), 270.0),
        (vec2(r.x + r.w - rad, r.y + r.h - rad), 0.0),
        (vec2(r.x + rad, r.y + r.h - rad), 90.0),
    ];
    let mut points = Vec::with_capacity(4 * (CORNER_SEGMENTS + 1));
    for (center, start) in corners {
        for i in 0..=CORNER_SEGMENTS {
            let a = (start + 90.0 * i as f32 / CORNER_SEGMENTS as f32).to_radians();
            points.push(center + vec2(a.cos(), a.sin()) * rad);
        }
    }
    points
}

/// 圓角矩形，上下兩色漸層
pub fn rounded_rect_gradient(r: Rect, radius: f32, top: Color, bottom: Color) {
    let color_at = |y: f32| {
        let t = ((y - r.y) / r.h).clamp(0.0, 1.0);
        Color::new(
            top.r + (bottom.r - top.r) * t,
            top.g + (bottom.g - top.g) * t,
            top.b + (bottom.b - top.b) * t,
            top.a + (bottom.a - top.a) * t,
        )
    };
    let center = r.center();
    let mut vertices = vec![Vertex::new(
        center.x,
        center.y,
        0.0,
        0.0,
        0.0,
        color_at(center.y),
    )];
    for p in outline(r, radius) {
        vertices.push(Vertex::new(p.x, p.y, 0.0, 0.0, 0.0, color_at(p.y)));
    }
    let n = vertices.len() as u16 - 1;
    let indices = (0..n).flat_map(|i| [0, 1 + i, 1 + (i + 1) % n]).collect();
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

/// 圓角矩形
pub fn rounded_rect(r: Rect, radius: f32, color: Color) {
    rounded_rect_gradient(r, radius, color, color);
}

/// 圓角矩形外框
pub fn rounded_rect_lines(r: Rect, radius: f32, thickness: f32, color: Color) {
    let points = outline(r, radius);
    for (i, a) in points.iter().enumerate() {
        let b = points[(i + 1) % points.len()];
        draw_line(a.x, a.y, b.x, b.y, thickness, color);
    }
}

/// 卡片下方的柔和陰影
pub fn shadow(r: Rect, radius: f32) {
    let offset = Rect::new(r.x, r.y + 3.0, r.w, r.h);
    rounded_rect(offset, radius, Color::new(0.0, 0.0, 0.0, 0.35));
}

/// 整個矩形的上下漸層（背景用）
pub fn gradient(r: Rect, top: Color, bottom: Color) {
    rounded_rect_gradient(r, 0.0, top, bottom);
}

/// 顏色變亮（amount > 0）或變暗（amount < 0），範圍 -1～1
pub fn shade(c: Color, amount: f32) -> Color {
    let f = |v: f32| {
        if amount >= 0.0 {
            v + (1.0 - v) * amount
        } else {
            v * (1.0 + amount)
        }
    };
    Color::new(f(c.r), f(c.g), f(c.b), c.a)
}

/// 同樣的顏色，換透明度
pub fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}
