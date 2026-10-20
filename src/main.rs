mod battle;
mod rng;
mod run;
mod sprite;

use battle::view::BattleScreen;
use macroquad::prelude::*;
use run::Run;

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
    let mut run = Run::new(1);
    let mut screen = BattleScreen::new(run.next_battle());

    loop {
        if is_key_pressed(KeyCode::Space) {
            screen = BattleScreen::new(run.next_battle());
        }
        screen.update(get_frame_time());

        clear_background(Color::new(0.15, 0.15, 0.18, 1.0));
        screen.draw();
        draw_text("Space: restart", 40.0, 100.0, 24.0, LIGHTGRAY);

        next_frame().await;
    }
}
