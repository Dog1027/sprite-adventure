//! 直式虛擬畫面：所有畫面都以 `W`×`H`（9:16）的座標排版，再等比縮放到視窗或手機螢幕，
//! 比例不同時兩側留黑邊。主要平台是手機，所以以直式為準。
//!
//! 每一幀：`begin` → 畫畫面 → `end`。滑鼠與觸控座標用 `to_canvas` 換算成虛擬座標
//! （`run::ui::mouse` 已經處理好，畫面程式不用自己換算）。

use macroquad::prelude::*;
use std::cell::RefCell;

/// 虛擬畫面的寬高
pub const W: f32 = 450.0;
pub const H: f32 = 800.0;

struct Canvas {
    target: RenderTarget,
    /// 虛擬畫面在視窗中的位置（視窗座標）
    dest: Rect,
}

thread_local! {
    static CANVAS: RefCell<Option<Canvas>> = const { RefCell::new(None) };
}

/// 每幀開始：依視窗大小決定虛擬畫面的位置，接下來的繪圖都畫在虛擬畫面上
pub fn begin() {
    let (sw, sh) = (screen_width(), screen_height());
    let scale = (sw / W).min(sh / H);
    let dest = Rect::new(
        ((sw - W * scale) / 2.0).floor(),
        ((sh - H * scale) / 2.0).floor(),
        W * scale,
        H * scale,
    );
    // 用實際像素大小建立畫布，縮放後文字與線條仍然清楚
    let dpi = screen_dpi_scale();
    let (pw, ph) = (
        ((dest.w * dpi).round() as u32).max(1),
        ((dest.h * dpi).round() as u32).max(1),
    );

    CANVAS.with(|c| {
        let mut c = c.borrow_mut();
        let reuse = c
            .as_ref()
            .is_some_and(|c| c.target.texture.size() == vec2(pw as f32, ph as f32));
        if !reuse {
            let target = render_target(pw, ph);
            target.texture.set_filter(FilterMode::Linear);
            *c = Some(Canvas { target, dest });
        }
        let canvas = c.as_mut().unwrap();
        canvas.dest = dest;

        let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, W, H));
        // 畫到畫布上時上下不用翻轉
        camera.zoom.y = -camera.zoom.y;
        camera.render_target = Some(canvas.target.clone());
        set_camera(&camera);
    });
}

/// 每幀結束：把虛擬畫面縮放畫到視窗上
pub fn end() {
    set_default_camera();
    clear_background(BLACK);
    CANVAS.with(|c| {
        if let Some(c) = c.borrow().as_ref() {
            draw_texture_ex(
                &c.target.texture,
                c.dest.x,
                c.dest.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(c.dest.size()),
                    ..Default::default()
                },
            );
        }
    });
}

/// 視窗座標換算成虛擬畫面座標
pub fn to_canvas(p: Vec2) -> Vec2 {
    CANVAS.with(|c| match c.borrow().as_ref() {
        Some(c) => (p - c.dest.point()) * (W / c.dest.w),
        None => p,
    })
}

/// 虛擬畫面的 1 單位是畫布上的幾個像素；文字用它決定點陣化的大小
pub fn pixel_scale() -> f32 {
    CANVAS.with(|c| match c.borrow().as_ref() {
        Some(c) => c.target.texture.width() / W,
        None => 1.0,
    })
}

/// 把目前的虛擬畫面存成 PNG（開發用截圖，見 `run::shots`）。要在 `begin` 之後、`end` 之前呼叫
#[cfg(not(target_arch = "wasm32"))]
pub fn save_png(path: &str) {
    // 先把這一幀已經排好的繪圖送出，再讀回畫布
    set_default_camera();
    CANVAS.with(|c| {
        if let Some(c) = c.borrow().as_ref() {
            let mut image = c.target.texture.get_texture_data();
            flip_rows(&mut image);
            image.export_png(path);
        }
    });
}

/// 讀回的畫布是由下往上存的，翻回來
#[cfg(not(target_arch = "wasm32"))]
fn flip_rows(image: &mut Image) {
    let row = image.width as usize * 4;
    let rows = image.height as usize;
    for y in 0..rows / 2 {
        let (top, bottom) = image.bytes.split_at_mut((rows - 1 - y) * row);
        top[y * row..(y + 1) * row].swap_with_slice(&mut bottom[..row]);
    }
}
