//! 戰鬥畫面：每隔一段時間進行一回合，畫出棋盤、攻擊線、血條與傷害數字。
//! 棋盤與精靈的畫法也提供給擺位畫面使用。

use super::{Battle, DEPTHS, Event, Outcome, Pos, ROWS, Side, Unit};
use macroquad::prelude::*;

pub const CELL: f32 = 80.0;
const BOARD_TOP: f32 = 160.0;
const ALLY_LEFT: f32 = 40.0;
const ENEMY_LEFT: f32 = 440.0;
const TURN_SECONDS: f32 = 0.8;

pub struct BattleScreen {
    pub battle: Battle,
    events: Vec<Event>,
    timer: f32,
}

impl BattleScreen {
    pub fn new(battle: Battle) -> Self {
        Self {
            battle,
            events: Vec::new(),
            timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.timer += dt;
        if self.timer >= TURN_SECONDS && self.battle.outcome.is_none() {
            self.timer = 0.0;
            self.events = self.battle.step();
        }
    }

    pub fn draw(&self) {
        let b = &self.battle;
        draw_text(format!("Turn {}", b.turn), 40.0, 60.0, 36.0, WHITE);
        if let Some(outcome) = b.outcome {
            let (text, color) = match outcome {
                Outcome::Win => ("WIN", GREEN),
                Outcome::Lose => ("LOSE", RED),
            };
            draw_text(text, 360.0, 500.0, 60.0, color);
        }

        draw_boards();

        for e in &self.events {
            if let Event::Attack {
                attacker,
                target,
                skill,
                ..
            } = *e
            {
                let color = if skill { GOLD } else { ORANGE };
                draw_link(&b.units[attacker], &b.units[target], 2.0, color);
            }
        }

        for u in &b.units {
            draw_unit(u);
        }

        for e in &self.events {
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
}

/// 格子左上角的畫面座標。我方右側為前方，敵方左側為前方。
pub fn cell_origin(side: Side, pos: Pos) -> Vec2 {
    let (left, col) = match side {
        Side::Ally => (ALLY_LEFT, (DEPTHS - 1 - pos.depth) as f32),
        Side::Enemy => (ENEMY_LEFT, pos.depth as f32),
    };
    vec2(left + col * CELL, BOARD_TOP + pos.row as f32 * CELL)
}

pub fn cell_center(side: Side, pos: Pos) -> Vec2 {
    cell_origin(side, pos) + vec2(CELL, CELL) / 2.0
}

/// 畫面座標落在哪一格
pub fn cell_at(side: Side, point: Vec2) -> Option<Pos> {
    for row in 0..ROWS {
        for depth in 0..DEPTHS {
            let pos = Pos { row, depth };
            let o = cell_origin(side, pos);
            if Rect::new(o.x, o.y, CELL, CELL).contains(point) {
                return Some(pos);
            }
        }
    }
    None
}

pub fn draw_boards() {
    for side in [Side::Ally, Side::Enemy] {
        for row in 0..ROWS {
            for depth in 0..DEPTHS {
                let o = cell_origin(side, Pos { row, depth });
                draw_rectangle_lines(o.x, o.y, CELL, CELL, 2.0, GRAY);
            }
        }
    }
}

pub fn draw_unit(u: &Unit) {
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

/// 兩隻精靈之間的連線（攻擊線、預覽線）
pub fn draw_link(from: &Unit, to: &Unit, thickness: f32, color: Color) {
    let a = cell_center(from.side, from.pos);
    let b = cell_center(to.side, to.pos);
    draw_line(a.x, a.y, b.x, b.y, thickness, color);
}

/// 擺位預覽線：標出每隻我方精靈的第一個攻擊目標，平手的候選以淡色表示
pub fn draw_preview(units: &[Unit]) {
    for (attacker, targets) in super::preview(units) {
        let color = if targets.len() == 1 {
            Color::new(1.0, 0.8, 0.2, 0.9)
        } else {
            Color::new(1.0, 0.8, 0.2, 0.4)
        };
        for t in targets {
            draw_link(&units[attacker], &units[t], 2.0, color);
        }
    }
}
