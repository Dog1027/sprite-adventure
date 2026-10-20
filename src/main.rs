mod battle;

use battle::{Battle, DEPTHS, Event, Outcome, Pos, ROWS, Side, Stats, Unit};
use macroquad::prelude::*;

const CELL: f32 = 80.0;
const BOARD_TOP: f32 = 160.0;
const ALLY_LEFT: f32 = 40.0;
const ENEMY_LEFT: f32 = 440.0;
const TURN_SECONDS: f32 = 0.8;

fn window_conf() -> Conf {
    Conf {
        window_title: "Sprite".to_owned(),
        window_width: 800,
        window_height: 600,
        high_dpi: true,
        ..Default::default()
    }
}

fn stats(
    hp: u32,
    attack: u32,
    defense: u32,
    interval: u32,
    start_turn: u32,
    skill_every: u32,
) -> Stats {
    Stats {
        hp,
        attack,
        defense,
        interval,
        start_turn,
        skill_every,
    }
}

// 示範用隊伍；之後改由資料檔與陣型畫面產生
fn sample_units() -> Vec<Unit> {
    let p = |row, depth| Pos { row, depth };
    vec![
        Unit::new("A1", Side::Ally, p(0, 0), stats(120, 30, 20, 1, 1, 3)),
        Unit::new("A2", Side::Ally, p(1, 1), stats(80, 50, 5, 2, 1, 2)),
        Unit::new("A3", Side::Ally, p(2, 0), stats(150, 20, 40, 1, 2, 4)),
        Unit::new("A4", Side::Ally, p(1, 3), stats(60, 70, 0, 3, 3, 2)),
        Unit::new("E1", Side::Enemy, p(0, 1), stats(100, 35, 10, 1, 1, 3)),
        Unit::new("E2", Side::Enemy, p(1, 0), stats(140, 25, 30, 1, 1, 3)),
        Unit::new("E3", Side::Enemy, p(2, 2), stats(70, 60, 0, 2, 2, 2)),
    ]
}

fn cell_origin(side: Side, pos: Pos) -> Vec2 {
    // 我方右側為前方，敵方左側為前方
    let col = match side {
        Side::Ally => (DEPTHS - 1 - pos.depth) as f32,
        Side::Enemy => pos.depth as f32,
    };
    vec2(
        match side {
            Side::Ally => ALLY_LEFT,
            Side::Enemy => ENEMY_LEFT,
        } + col * CELL,
        BOARD_TOP + pos.row as f32 * CELL,
    )
}

fn draw_board(left: f32) {
    for r in 0..ROWS {
        for c in 0..DEPTHS {
            let x = left + c as f32 * CELL;
            let y = BOARD_TOP + r as f32 * CELL;
            draw_rectangle_lines(x, y, CELL, CELL, 2.0, GRAY);
        }
    }
}

fn draw_battle(b: &Battle, events: &[Event]) {
    draw_board(ALLY_LEFT);
    draw_board(ENEMY_LEFT);

    for e in events {
        if let Event::Attack {
            attacker,
            target,
            skill,
            ..
        } = *e
        {
            let from =
                cell_origin(b.units[attacker].side, b.units[attacker].pos) + vec2(CELL, CELL) / 2.0;
            let to =
                cell_origin(b.units[target].side, b.units[target].pos) + vec2(CELL, CELL) / 2.0;
            draw_line(
                from.x,
                from.y,
                to.x,
                to.y,
                2.0,
                if skill { GOLD } else { ORANGE },
            );
        }
    }

    for u in &b.units {
        let o = cell_origin(u.side, u.pos);
        let color = match (u.alive(), u.side) {
            (false, _) => Color::new(0.3, 0.3, 0.3, 1.0),
            (true, Side::Ally) => SKYBLUE,
            (true, Side::Enemy) => PINK,
        };
        draw_rectangle(o.x + 8.0, o.y + 8.0, CELL - 16.0, CELL - 32.0, color);
        draw_text(&u.name, o.x + 14.0, o.y + 34.0, 24.0, BLACK);

        let ratio = u.hp as f32 / u.stats.hp as f32;
        draw_rectangle(o.x + 8.0, o.y + CELL - 20.0, CELL - 16.0, 8.0, DARKGRAY);
        draw_rectangle(
            o.x + 8.0,
            o.y + CELL - 20.0,
            (CELL - 16.0) * ratio,
            8.0,
            GREEN,
        );
    }

    for e in events {
        if let Event::Attack {
            target,
            damage,
            skill,
            ..
        } = *e
        {
            let o = cell_origin(b.units[target].side, b.units[target].pos);
            let label = if skill {
                format!("-{damage}!")
            } else {
                format!("-{damage}")
            };
            draw_text(&label, o.x + CELL - 34.0, o.y + 22.0, 22.0, RED);
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut seed = 1;
    let mut battle = Battle::new(sample_units(), seed);
    let mut events = Vec::new();
    let mut timer = 0.0;

    loop {
        if is_key_pressed(KeyCode::Space) {
            seed += 1;
            battle = Battle::new(sample_units(), seed);
            events.clear();
            timer = 0.0;
        }

        timer += get_frame_time();
        if timer >= TURN_SECONDS && battle.outcome.is_none() {
            timer = 0.0;
            events = battle.step();
        }

        clear_background(Color::new(0.15, 0.15, 0.18, 1.0));
        draw_text(format!("Turn {}", battle.turn), 40.0, 60.0, 36.0, WHITE);
        draw_text("Space: restart", 40.0, 100.0, 24.0, LIGHTGRAY);
        if let Some(outcome) = battle.outcome {
            let (text, color) = match outcome {
                Outcome::Win => ("WIN", GREEN),
                Outcome::Lose => ("LOSE", RED),
            };
            draw_text(text, 360.0, 500.0, 60.0, color);
        }
        draw_battle(&battle, &events);

        next_frame().await;
    }
}
