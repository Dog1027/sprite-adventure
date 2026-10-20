//! 戰鬥畫面：每隔一段時間進行一回合，畫出棋盤、攻擊線、血條與傷害數字。
//! 棋盤與精靈的畫法也提供給擺位畫面使用。
//!
//! 直式排版：敵方棋盤在上、我方在下，雙方的前排都朝向中間。
//! 「列」（`row`）是直的，左到右是第 1～3 列，上下對齊，所以「同列」就是正對面。

use super::{Battle, DEPTHS, Event, Pos, ROWS, Side, Unit};
use crate::gfx;
use crate::palette;
use crate::screen::W;
use crate::text;
use macroquad::prelude::*;

/// 一格的寬高
pub const CELL_W: f32 = 120.0;
pub const CELL_H: f32 = 58.0;
const BOARD_LEFT: f32 = (W - CELL_W * ROWS as f32) / 2.0;
const BOARD_H: f32 = CELL_H * DEPTHS as f32;
/// 敵方棋盤的上緣
pub const ENEMY_TOP: f32 = 124.0;
/// 兩個棋盤中間的空隙（前線）
const FRONT_GAP: f32 = 20.0;
const ALLY_TOP: f32 = ENEMY_TOP + BOARD_H + FRONT_GAP;
/// 我方棋盤的下緣，下面可以放其他介面
pub const BOARD_BOTTOM: f32 = ALLY_TOP + BOARD_H;

/// 一回合幾秒（一般、快轉）
const TURN_SECONDS: f32 = 0.8;
const FAST_TURN_SECONDS: f32 = 0.3;

pub struct BattleScreen {
    pub battle: Battle,
    /// 快轉
    pub fast: bool,
    events: Vec<Event>,
    timer: f32,
}

impl BattleScreen {
    pub fn new(battle: Battle) -> Self {
        Self {
            battle,
            fast: false,
            events: Vec::new(),
            timer: 0.0,
        }
    }

    fn turn_seconds(&self) -> f32 {
        if self.fast {
            FAST_TURN_SECONDS
        } else {
            TURN_SECONDS
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.timer += dt;
        if self.timer >= self.turn_seconds() && self.battle.outcome.is_none() {
            self.timer = 0.0;
            self.events = self.battle.step();
        }
    }

    /// 直接打到分出勝負
    pub fn skip(&mut self) {
        while self.battle.outcome.is_none() {
            self.events = self.battle.step();
        }
        self.timer = self.turn_seconds();
    }

    pub fn draw(&self) {
        let b = &self.battle;
        // 這一回合的動畫進度 0～1
        let t = (self.timer / self.turn_seconds()).min(1.0);
        draw_boards();

        for e in &self.events {
            let (from, to, color) = match *e {
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
                    (attacker, target, color)
                }
                Event::Heal { healer, target, .. } => (healer, target, palette::HEAL),
                Event::Defeated { .. } => continue,
            };
            draw_link(
                &b.units[from],
                &b.units[to],
                3.0,
                gfx::with_alpha(color, 1.0 - t * 0.6),
            );
        }

        for u in &b.units {
            draw_unit(u, None);
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
            let r = tile_rect(&b.units[target]);
            // 被打的格子閃一下
            gfx::rounded_rect(r, 8.0, gfx::with_alpha(palette::DAMAGE, 0.35 * (1.0 - t)));
            let label = if skill {
                format!("-{damage}!")
            } else {
                format!("-{damage}")
            };
            // 傷害數字靠右，往上飄
            let y = r.y + 26.0 - t * 8.0;
            text::right(&label, r.x + r.w - 8.0, y, 18.0, palette::DAMAGE);
        }
        for (target, amount) in heal_totals {
            let r = tile_rect(&b.units[target]);
            draw_tag(
                &format!("+{amount}"),
                vec2(r.center().x, r.y + r.h - 2.0 - t * 6.0),
                palette::HEAL,
            );
        }
        for caster in casters {
            let r = tile_rect(&b.units[caster]);
            draw_tag(
                &format!("「{}」", b.units[caster].skill.name),
                vec2(r.center().x, r.y),
                palette::SKILL,
            );
        }
    }
}

/// 有深色底的小標籤，以 `center` 為中心
fn draw_tag(label: &str, center: Vec2, color: Color) {
    let w = text::measure(label, 12.0).width + 12.0;
    let r = Rect::new(center.x - w / 2.0, center.y - 9.0, w, 18.0);
    gfx::rounded_rect(r, 9.0, palette::TAG_BG);
    text::centered(label, center, 12.0, color);
}

/// 格子左上角的畫面座標。敵方的前排（depth 0）在下，我方的前排在上。
pub fn cell_origin(side: Side, pos: Pos) -> Vec2 {
    let x = BOARD_LEFT + pos.row as f32 * CELL_W;
    let y = match side {
        Side::Enemy => ENEMY_TOP + (DEPTHS - 1 - pos.depth) as f32 * CELL_H,
        Side::Ally => ALLY_TOP + pos.depth as f32 * CELL_H,
    };
    vec2(x, y)
}

pub fn cell_rect(side: Side, pos: Pos) -> Rect {
    let o = cell_origin(side, pos);
    Rect::new(o.x, o.y, CELL_W, CELL_H)
}

/// 精靈方塊的範圍（格子往內縮一點）
fn tile_rect(u: &Unit) -> Rect {
    inset(cell_rect(u.side, u.pos), 3.0)
}

pub fn inset(r: Rect, by: f32) -> Rect {
    Rect::new(r.x + by, r.y + by, r.w - 2.0 * by, r.h - 2.0 * by)
}

/// 畫面座標落在哪一格
pub fn cell_at(side: Side, point: Vec2) -> Option<Pos> {
    (0..ROWS)
        .flat_map(|row| (0..DEPTHS).map(move |depth| Pos { row, depth }))
        .find(|&pos| cell_rect(side, pos).contains(point))
}

/// 雙方棋盤：底色、格線、左側的「敵方／我方」、中間的前線
pub fn draw_boards() {
    for (side, top, tint, label) in [
        (Side::Enemy, ENEMY_TOP, palette::BOARD_ENEMY, ["敵", "方"]),
        (Side::Ally, ALLY_TOP, palette::BOARD_ALLY, ["我", "方"]),
    ] {
        let board = Rect::new(BOARD_LEFT, top, CELL_W * ROWS as f32, BOARD_H);
        gfx::rounded_rect(board, 12.0, tint);
        for row in 0..ROWS {
            for depth in 0..DEPTHS {
                let r = inset(cell_rect(side, Pos { row, depth }), 1.0);
                gfx::rounded_rect_lines(r, 8.0, 1.0, palette::GRID);
            }
        }
        let color = match side {
            Side::Ally => palette::ALLY,
            Side::Enemy => palette::ENEMY,
        };
        let mid = top + BOARD_H / 2.0;
        for (i, ch) in label.iter().enumerate() {
            text::centered(
                ch,
                vec2(BOARD_LEFT / 2.0, mid - 12.0 + i as f32 * 24.0),
                18.0,
                gfx::with_alpha(color, 0.7),
            );
        }
    }
    // 前線：虛線
    let y = ENEMY_TOP + BOARD_H + FRONT_GAP / 2.0;
    let mut x = BOARD_LEFT;
    while x < BOARD_LEFT + CELL_W * ROWS as f32 {
        draw_line(x, y, x + 8.0, y, 1.0, palette::TEXT_MUTED);
        x += 14.0;
    }
}

/// 畫一隻精靈。`footer` 會畫在左下角並取代血量數字（擺位畫面用來顯示陣型加成）
pub fn draw_unit(u: &Unit, footer: Option<&str>) {
    let r = tile_rect(u);
    if !u.alive() {
        gfx::rounded_rect(r, 8.0, gfx::with_alpha(palette::DEFEATED, 0.6));
        text::draw(&u.name, r.x + 10.0, r.y + 28.0, 24.0, palette::TEXT_MUTED);
        return;
    }
    let side_color = match u.side {
        Side::Ally => palette::ALLY,
        Side::Enemy => palette::ENEMY,
    };
    gfx::rounded_rect_gradient(
        r,
        8.0,
        gfx::shade(side_color, -0.55),
        gfx::shade(side_color, -0.7),
    );
    gfx::rounded_rect_lines(r, 8.0, 2.0, gfx::with_alpha(side_color, 0.9));
    text::draw(&u.name, r.x + 10.0, r.y + 28.0, 24.0, palette::TEXT);

    let ratio = u.hp as f32 / u.stats.hp as f32;
    let bar = Rect::new(r.x + 8.0, r.y + 35.0, r.w - 16.0, 7.0);
    gfx::rounded_rect(bar, 3.5, palette::HP_BAR_BG);
    if ratio > 0.0 {
        let fill = if ratio < 0.3 {
            palette::HP_BAR_LOW
        } else {
            palette::HP_BAR
        };
        gfx::rounded_rect(
            Rect::new(bar.x, bar.y, (bar.w * ratio).max(bar.h), bar.h),
            3.5,
            fill,
        );
    }
    match footer {
        Some(label) => text::draw(label, r.x + 8.0, r.y + r.h - 3.0, 12.0, palette::HIGHLIGHT),
        None => text::right(
            &format!("{}/{}", u.hp, u.stats.hp),
            r.x + r.w - 8.0,
            r.y + r.h - 3.0,
            12.0,
            palette::TEXT_DIM,
        ),
    }
}

/// 線的端點：朝向對方的那一邊（我方往上、敵方往下）；同一方之間用中心
fn anchor(u: &Unit, other: &Unit) -> Vec2 {
    let r = tile_rect(u);
    if u.side == other.side {
        return r.center();
    }
    match u.side {
        Side::Ally => vec2(r.center().x, r.y),
        Side::Enemy => vec2(r.center().x, r.y + r.h),
    }
}

/// 兩隻精靈之間的連線（攻擊線、預覽線），目標那端畫一個圓點
pub fn draw_link(from: &Unit, to: &Unit, thickness: f32, color: Color) {
    let a = anchor(from, to);
    let b = anchor(to, from);
    draw_line(a.x, a.y, b.x, b.y, thickness, color);
    draw_circle(b.x, b.y, thickness + 2.0, color);
}

/// 擺位預覽線：標出每隻我方精靈的第一個攻擊目標，平手的候選以淡色表示
pub fn draw_preview(units: &[Unit]) {
    for (attacker, targets) in super::preview(units) {
        let alpha = if targets.len() == 1 { 0.9 } else { 0.45 };
        for &t in &targets {
            draw_link(
                &units[attacker],
                &units[t],
                2.0,
                gfx::with_alpha(palette::PREVIEW_LINE, alpha),
            );
        }
        let names: Vec<&str> = targets.iter().map(|&t| units[t].name.as_str()).collect();
        let r = tile_rect(&units[attacker]);
        text::right(
            &format!("→{}", names.join("/")),
            r.x + r.w - 8.0,
            r.y + 18.0,
            12.0,
            palette::PREVIEW_LINE,
        );
    }
}
