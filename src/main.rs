mod battle;
mod rng;
mod run;
mod sprite;

use macroquad::prelude::*;
use run::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Sprite".to_owned(),
        window_width: 800,
        window_height: 600,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new(1);

    loop {
        clear_background(Color::new(0.15, 0.15, 0.18, 1.0));
        game.frame(get_frame_time());
        next_frame().await;
    }
}
