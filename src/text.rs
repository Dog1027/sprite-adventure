//! 文字繪製：使用支援中文的字型（assets/fonts/Cubic_11.ttf，俐方體11號，SIL OFL 1.1）。
//! 所有畫面的文字都透過這裡畫，不要直接呼叫 macroquad 的 `draw_text`。

use crate::screen;
use macroquad::prelude::*;
use std::cell::OnceCell;

const FONT_PATH: &str = "assets/fonts/Cubic_11.ttf";

thread_local! {
    static FONT: OnceCell<Font> = const { OnceCell::new() };
}

/// 遊戲開始前載入字型；失敗時退回預設字型（中文會無法顯示）
pub async fn load() {
    match load_ttf_font(FONT_PATH).await {
        Ok(mut font) => {
            // 字會依畫布像素點陣化後再縮放，用線性過濾比較平滑
            font.set_filter(FilterMode::Linear);
            FONT.with(|f| {
                let _ = f.set(font);
            });
        }
        Err(e) => {
            error!("無法載入字型 {FONT_PATH}：{e}");
        }
    }
}

/// 點陣化用的字級與縮放：依畫布的實際像素放大點陣化，再縮回虛擬畫面的大小，手機上才清楚
fn raster(size: f32) -> (u16, f32) {
    let px = (size * screen::pixel_scale()).round().max(1.0);
    (px as u16, size / px)
}

/// 以左下角為基準畫文字（虛擬畫面座標）。字型以 12px 設計，大小用 12 的倍數最清楚。
pub fn draw(text: &str, x: f32, y: f32, size: f32, color: Color) {
    let (font_size, font_scale) = raster(size);
    FONT.with(|f| {
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: f.get(),
                font_size,
                font_scale,
                color,
                ..Default::default()
            },
        );
    });
}

pub fn measure(text: &str, size: f32) -> TextDimensions {
    let (font_size, font_scale) = raster(size);
    FONT.with(|f| measure_text(text, f.get(), font_size, font_scale))
}

/// 靠右對齊，`right` 是文字右緣
pub fn right(text: &str, right: f32, y: f32, size: f32, color: Color) {
    draw(text, right - measure(text, size).width, y, size, color);
}

/// 以中心點畫文字
pub fn centered(text: &str, center: Vec2, size: f32, color: Color) {
    let m = measure(text, size);
    draw(
        text,
        center.x - m.width / 2.0,
        center.y + m.offset_y / 2.0,
        size,
        color,
    );
}
