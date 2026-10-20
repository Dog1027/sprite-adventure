//! 戰鬥畫面：每隔一段時間進行一回合，畫出棋盤、攻擊線、血條與傷害數字。
//! 棋盤與精靈的畫法也提供給擺位畫面使用。

use super::{Battle, DEPTHS, Event, Outcome, Pos, ROWS, Side, Unit};
use crate::palette;
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
        text::draw(
            &format!("第 {} 回合", b.turn),
            40.0,
            60.0,
            36.0,
            palette::TEXT,
        );
        if let Some(outcome) = b.outcome {
            let (label, color) = match outcome {
                Outcome::Win => ("勝利", palette::SUCCESS),
                Outcome::Lose => ("戰敗", palette::FAILURE),
            };
            text::centered(label, vec2(400.0, 480.0), 48.0, color);
        }

        draw_boards();

        for e in &self.events {
            match *e {
                Event::Attack {
                    attacker,
                    target,
                    skill,
                    ..
                } => {
                    let color = if skill {
                        palette::SKILL
                    } else {
                        palette::ATTACK_LINE
                    };
                    draw_link(&b.units[attacker], &b.units[target], 2.0, color);
                }
                Event::Heal { healer, target, .. } => {
                    draw_link(&b.units[healer], &b.units[target], 2.0, palette::HEAL);
                }
                Event::Defeated { .. } => {}
            }
        }

        for u in &b.units {
            draw_unit(u);
        }

        // 同一回合被打或被治療多次時合計，避免數字重疊
        let mut damage_totals: Vec<(usize, u32, bool)> = Vec::new();
        let mut heal_totals: Vec<(usize, u32)> = Vec::new();
        let mut casters: Vec<usize> = Vec::new();
        for e in &self.events {
            match *e {
                Event::Attack {
                    attacker,
                    target,
                    damage,
                    skill,
                } => {
                    match damage_totals.iter_mut().find(|t| t.0 == target) {
                        Some(t) => {
                            t.1 += damage;
                            t.2 |= skill;
                        }
                        None => damage_totals.push((target, damage, skill)),
                    }
                    if skill && !casters.contains(&attacker) {
                        casters.push(attacker);
                    }
                }
                Event::Heal {
                    healer,
                    target,
                    amount,
                } => {
                    match heal_totals.iter_mut().find(|t| t.0 == target) {
                        Some(t) => t.1 += amount,
                        None => heal_totals.push((target, amount)),
                    }
                    if !casters.contains(&healer) {
                        casters.push(healer);
                    }
                }
                Event::Defeated { .. } => {}
            }
        }
        for (target, damage, skill) in damage_totals {
            let label = if skill {
                format!("-{damage}!")
            } else {
                format!("-{damage}")
            };
            // 傷害靠右畫在名字下方
            draw_right_aligned(&b.units[target], &label, 54.0, palette::DAMAGE);
        }
        for (target, amount) in heal_totals {
            // 治療量畫在格子下緣，和上緣的技能名稱同樣用標籤
            let u = &b.units[target];
            let x = cell_center(u.side, u.pos).x;
            draw_tag(
                &format!("+{amount}"),
                vec2(x, cell_origin(u.side, u.pos).y + CELL - 4.0),
                palette::HEAL,
            );
        }
        for caster in casters {
            draw_skill_name(&b.units[caster]);
        }
    }
}

fn draw_right_aligned(u: &Unit, label: &str, y: f32, color: Color) {
    let o = cell_origin(u.side, u.pos);
    let w = text::measure(label, 24.0).width;
    text::draw(label, o.x + CELL - 10.0 - w, o.y + y, 24.0, color);
}

/// 施放技能時，在精靈上方顯示技能名稱
fn draw_skill_name(u: &Unit) {
    let x = cell_center(u.side, u.pos).x;
    let top = cell_origin(u.side, u.pos).y;
    draw_tag(
        &format!("「{}」", u.skill.name),
        vec2(x, top + 2.0),
        palette::SKILL,
    );
}

/// 有深色底的小標籤，以 `center` 為中心
fn draw_tag(label: &str, center: Vec2, color: Color) {
    let w = text::measure(label, 12.0).width + 8.0;
    draw_rectangle(center.x - w / 2.0, center.y - 8.0, w, 16.0, palette::TAG_BG);
    text::centered(label, center, 12.0, color);
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
                draw_rectangle_lines(o.x, o.y, CELL, CELL, 2.0, palette::GRID);
            }
        }
    }
}

pub fn draw_unit(u: &Unit) {
    let o = cell_origin(u.side, u.pos);
    let color = match (u.alive(), u.side) {
        (false, _) => palette::DEFEATED,
        (true, Side::Ally) => palette::ALLY,
        (true, Side::Enemy) => palette::ENEMY,
    };
    draw_rectangle(o.x + 8.0, o.y + 8.0, CELL - 16.0, CELL - 32.0, color);
    text::draw(
        &u.name,
        o.x + 12.0,
        o.y + 32.0,
        24.0,
        palette::TEXT_ON_LIGHT,
    );

    let ratio = u.hp as f32 / u.stats.hp as f32;
    draw_rectangle(
        o.x + 8.0,
        o.y + CELL - 20.0,
        CELL - 16.0,
        8.0,
        palette::HP_BAR_BG,
    );
    draw_rectangle(
        o.x + 8.0,
        o.y + CELL - 20.0,
        (CELL - 16.0) * ratio,
        8.0,
        palette::HP_BAR,
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
                Color {
                    a: alpha,
                    ..palette::PREVIEW_LINE
                },
            );
        }
        let names: Vec<&str> = targets.iter().map(|&t| units[t].name.as_str()).collect();
        let o = cell_origin(units[attacker].side, units[attacker].pos);
        text::draw(
            &format!("→{}", names.join("/")),
            o.x + 12.0,
            o.y + 50.0,
            12.0,
            palette::TEXT_ON_LIGHT,
        );
    }
}
