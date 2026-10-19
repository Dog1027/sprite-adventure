use macroquad::prelude::*;

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
    let mut pos = vec2(screen_width() / 2.0, screen_height() / 2.0);
    let speed = 300.0;

    loop {
        let dt = get_frame_time();
        let mut dir = Vec2::ZERO;
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { dir.x += 1.0; }
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) { dir.x -= 1.0; }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) { dir.y += 1.0; }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) { dir.y -= 1.0; }
        pos += dir.normalize_or_zero() * speed * dt;
        pos = pos.clamp(Vec2::ZERO, vec2(screen_width(), screen_height()));

        clear_background(DARKGRAY);
        draw_circle(pos.x, pos.y, 20.0, YELLOW);
        draw_text("WASD / 方向鍵移動", 20.0, 30.0, 24.0, WHITE);
        draw_text(&format!("FPS: {}", get_fps()), 20.0, 56.0, 24.0, LIGHTGRAY);

        next_frame().await;
    }
}
