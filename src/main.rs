mod battle;
mod gfx;
mod palette;
mod rng;
mod run;
mod screen;
mod sprite;
mod text;

use macroquad::prelude::*;
use run::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Sprite".to_owned(),
        // 直式，主要平台是手機；電腦上的視窗也用同樣比例
        window_width: screen::W as i32,
        window_height: screen::H as i32,
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    text::load().await;

    // 開發用：SPRITE_SHOTS=資料夾 會把每個畫面存成 PNG 後結束（見 `just shots`）
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(dir) = std::env::var("SPRITE_SHOTS") {
        run::shots::save_all(&dir).await;
        std::process::exit(0);
    }

    let mut game = Game::new(1);
    loop {
        screen::begin();
        game.frame(get_frame_time());
        screen::end();
        next_frame().await;
    }
}
