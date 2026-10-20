//! 文字繪製：使用支援中文的字型（assets/fonts/Cubic_11.ttf，俐方體11號，SIL OFL 1.1）。
//! 所有畫面的文字都透過這裡畫，不要直接呼叫 macroquad 的 `draw_text`。

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
            // 點陣字型，放大時保持銳利
            font.set_filter(FilterMode::Nearest);
            FONT.with(|f| {
                let _ = f.set(font);
            });
        }
        Err(e) => {
            error!("無法載入字型 {FONT_PATH}：{e}");
        }
    }
}

/// 以左下角為基準畫文字。字型以 12px 設計，大小用 12 的倍數最清楚。
pub fn draw(text: &str, x: f32, y: f32, size: f32, color: Color) {
    FONT.with(|f| {
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: f.get(),
                font_size: size as u16,
                color,
                ..Default::default()
            },
        );
    });
}

pub fn measure(text: &str, size: f32) -> TextDimensions {
    FONT.with(|f| measure_text(text, f.get(), size as u16, 1.0))
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
