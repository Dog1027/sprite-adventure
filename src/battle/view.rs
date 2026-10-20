//! 戰鬥畫面：每隔一段時間進行一回合，畫出棋盤、攻擊線、血條與傷害數字。
//! 棋盤與精靈的畫法也提供給擺位畫面使用。

use super::{Battle, DEPTHS, Event, Outcome, Pos, ROWS, Side, Unit};
use crate::text;
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
        text::draw(&format!("第 {} 回合", b.turn), 40.0, 60.0, 36.0, WHITE);
        if let Some(outcome) = b.outcome {
            let (label, color) = match outcome {
                Outcome::Win => ("勝利", GREEN),
                Outcome::Lose => ("戰敗", RED),
            };
            text::centered(label, vec2(400.0, 480.0), 48.0, color);
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

        // 同一回合被打多次時合計傷害，避免數字重疊
        let mut totals: Vec<(usize, u32, bool)> = Vec::new();
        for e in &self.events {
            if let Event::Attack {
                target,
                damage,
                skill,
                ..
            } = *e
            {
                match totals.iter_mut().find(|t| t.0 == target) {
                    Some(t) => {
                        t.1 += damage;
                        t.2 |= skill;
                    }
                    None => totals.push((target, damage, skill)),
                }
            }
        }
        for (target, damage, skill) in totals {
            let o = cell_origin(b.units[target].side, b.units[target].pos);
            let label = if skill {
                format!("-{damage}!")
            } else {
                format!("-{damage}")
            };
            // 靠右畫在名字下方
            let w = text::measure(&label, 24.0).width;
            text::draw(&label, o.x + CELL - 10.0 - w, o.y + 54.0, 24.0, RED);
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
    text::draw(&u.name, o.x + 12.0, o.y + 32.0, 24.0, BLACK);

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

/// 擺位預覽線：標出每隻我方精靈的第一個攻擊目標（畫在精靈上方），平手的候選以淡色表示
pub fn draw_preview(units: &[Unit]) {
    for (attacker, targets) in super::preview(units) {
        let alpha = if targets.len() == 1 { 0.9 } else { 0.45 };
        for &t in &targets {
            draw_link(
                &units[attacker],
                &units[t],
                2.0,
                Color::new(1.0, 0.8, 0.2, alpha),
            );
        }
        let names: Vec<&str> = targets.iter().map(|&t| units[t].name.as_str()).collect();
        let o = cell_origin(units[attacker].side, units[attacker].pos);
        text::draw(
            &format!("→{}", names.join("/")),
            o.x + 12.0,
            o.y + 50.0,
            12.0,
            BLACK,
        );
    }
}
