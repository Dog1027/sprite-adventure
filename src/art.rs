//! 精靈動畫圖：`assets/sprites/<代號>.png`（代號是 `data/species.toml` 的 key）。
//! 目前是 `tools/gen_placeholder_sprites.py` 產生的占位圖，正式美術用同樣排法蓋掉即可。
//!
//! 圖的排法：每格 32×32 像素，每一列是一種動作，由左到右播放（見 `Anim`）。
//! 角色面向右邊，敵方畫的時候左右翻轉。腳底在每格的最下緣。
//! 沒有圖的精靈 `draw` 會回傳 false，畫面改畫原本的名字牌。

use crate::sprite::species;
use macroquad::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

/// 每格的像素大小
const FRAME_PX: f32 = 32.0;
/// 畫面上放大幾倍（像素風用整數倍）
const SCALE: f32 = 2.0;
/// 畫在畫面上的大小（虛擬畫面座標）
pub const SIZE: f32 = FRAME_PX * SCALE;
/// 待機動畫每秒幾格
const IDLE_FPS: f32 = 4.0;

/// 動作；順序就是圖中列的順序
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anim {
    /// 循環播放
    Idle,
    Attack,
    Skill,
    Hit,
    /// 播完停在最後一格
    Defeated,
}

impl Anim {
    fn row(self) -> usize {
        self as usize
    }

    fn frames(self) -> usize {
        match self {
            Anim::Hit => 2,
            _ => 4,
        }
    }
}

/// 動作進度 0～1 對應到第幾格
fn frame_index(anim: Anim, progress: f32) -> usize {
    let n = anim.frames();
    ((progress.clamp(0.0, 1.0) * n as f32) as usize).min(n - 1)
}

/// 待機動畫目前的進度；`phase` 0～1 讓每隻精靈錯開，不會整齊劃一
pub fn idle_progress(phase: f32) -> f32 {
    (get_time() as f32 * IDLE_FPS / Anim::Idle.frames() as f32 + phase).fract()
}

thread_local! {
    static SHEETS: RefCell<HashMap<&'static str, Texture2D>> = RefCell::new(HashMap::new());
}

/// 遊戲開始前載入所有精靈的圖；缺少的圖只記錄警告
pub async fn load() {
    for id in species::ids() {
        let path = format!("assets/sprites/{id}.png");
        match load_texture(&path).await {
            Ok(tex) => {
                tex.set_filter(FilterMode::Nearest);
                SHEETS.with(|s| s.borrow_mut().insert(id, tex));
            }
            Err(e) => warn!("無法載入精靈圖 {path}：{e}"),
        }
    }
}

/// 這隻精靈有沒有動畫圖
pub fn has(species: &str) -> bool {
    SHEETS.with(|s| s.borrow().contains_key(species))
}

/// 畫一格動畫。`foot` 是腳底中心（虛擬畫面座標），`progress` 是動作進度 0～1，
/// `flip` 左右翻轉（敵方），`color` 用來調整透明度或染色。沒有這隻精靈的圖時回傳 false
pub fn draw(
    species: &str,
    anim: Anim,
    progress: f32,
    foot: Vec2,
    flip: bool,
    color: Color,
) -> bool {
    SHEETS.with(|s| {
        let sheets = s.borrow();
        let Some(tex) = sheets.get(species) else {
            return false;
        };
        let col = frame_index(anim, progress) as f32;
        let row = anim.row() as f32;
        draw_texture_ex(
            tex,
            (foot.x - SIZE / 2.0).round(),
            (foot.y - SIZE).round(),
            color,
            DrawTextureParams {
                dest_size: Some(vec2(SIZE, SIZE)),
                source: Some(Rect::new(
                    col * FRAME_PX,
                    row * FRAME_PX,
                    FRAME_PX,
                    FRAME_PX,
                )),
                flip_x: flip,
                ..Default::default()
            },
        );
        true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_maps_to_frames() {
        assert_eq!(frame_index(Anim::Attack, 0.0), 0);
        assert_eq!(frame_index(Anim::Attack, 0.3), 1);
        assert_eq!(frame_index(Anim::Attack, 0.99), 3);
        assert_eq!(frame_index(Anim::Attack, 1.0), 3);
        assert_eq!(frame_index(Anim::Hit, 0.6), 1);
        assert_eq!(frame_index(Anim::Defeated, 5.0), 3);
        assert_eq!(frame_index(Anim::Idle, -1.0), 0);
    }

    #[test]
    fn rows_follow_sheet_layout() {
        let rows = [
            Anim::Idle,
            Anim::Attack,
            Anim::Skill,
            Anim::Hit,
            Anim::Defeated,
        ]
        .map(Anim::row);
        assert_eq!(rows, [0, 1, 2, 3, 4]);
    }

    #[test]
    fn every_species_has_a_sheet() {
        for id in species::ids() {
            let path = format!("{}/assets/sprites/{id}.png", env!("CARGO_MANIFEST_DIR"));
            assert!(std::path::Path::new(&path).exists(), "缺少 {path}");
        }
    }
}
