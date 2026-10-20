//! 戰鬥畫面：每隔一段時間進行一回合，畫出棋盤、攻擊線、血條與傷害數字。
//! 棋盤與精靈的畫法也提供給擺位畫面使用。
//!
//! 斜向排版：我方在左下、敵方在右上，戰場往右上傾斜 `TILT_DEG` 度，雙方的前排朝向中間。
//! 每一格在地上是平行四邊形；精靈像立牌一樣站在格子上，由後往前畫，
//! 前面的精靈會稍微蓋住後面的，有立體感（之後換成精靈動畫也照這個順序畫）。
//! 「列」（`row`）是左到右的第 1～3 列，沿著戰場方向對齊，所以「同列」就是正對面。

use super::{Battle, DEPTHS, Event, Pos, ROWS, Side, Unit};
use crate::gfx;
use crate::palette;
use crate::screen::W;
use crate::text;
use macroquad::prelude::*;

/// 戰場往右上傾斜的角度（從正上方往右算）
const TILT_DEG: f32 = 22.0;
/// 一行（前後一格）沿著戰場方向的長度
const ROW_DEPTH: f32 = 58.0;
/// 兩邊棋盤中間的空隙（前線）
const FRONT_GAP: f32 = 24.0;
/// 整個戰場沿著戰場方向的長度：我方 4 行＋前線＋敵方 4 行
const FIELD_LEN: f32 = ROW_DEPTH * (2 * DEPTHS) as f32 + FRONT_GAP;
/// 戰場左右留白
const FIELD_MARGIN: f32 = 16.0;
/// 戰場最下緣（我方最後一行）的高度
const FIELD_BOTTOM: f32 = 600.0;
/// 戰場下方可以放其他介面的位置
pub const BOARD_BOTTOM: f32 = FIELD_BOTTOM + 8.0;
/// 精靈立牌的大小：比格子寬一點，左右會稍微重疊
const UNIT_EXTRA_W: f32 = 10.0;
const UNIT_H: f32 = 48.0;
/// 立牌下緣比格子中心低多少（看起來站在格子上）
const UNIT_FOOT: f32 = 10.0;

/// 一回合幾秒（一般、快轉）
const TURN_SECONDS: f32 = 0.8;
const FAST_TURN_SECONDS: f32 = 0.3;

/// 往前（朝敵方）的方向，畫面座標的單位向量
fn forward() -> Vec2 {
    let a = TILT_DEG.to_radians();
    vec2(a.sin(), -a.cos())
}

/// 一列的寬度：讓整個戰場剛好放進畫面寬度
fn lane_w() -> f32 {
    (W - 2.0 * FIELD_MARGIN - FIELD_LEN * forward().x) / ROWS as f32
}

/// 戰場座標的原點：我方最後一行下緣的中間
fn origin() -> Vec2 {
    vec2(W / 2.0 - FIELD_LEN * forward().x / 2.0, FIELD_BOTTOM)
}

/// 戰場座標 → 畫面座標。`v` 是左右（列的方向），`u` 是沿著戰場往前的距離
fn to_screen(v: f32, u: f32) -> Vec2 {
    origin() + vec2(v, 0.0) + forward() * u
}

/// 畫面座標 → 戰場座標 (v, u)
fn to_field(p: Vec2) -> (f32, f32) {
    let d = p - origin();
    let f = forward();
    let u = d.y / f.y;
    (d.x - u * f.x, u)
}

/// 這一行中心沿著戰場方向的距離
fn depth_u(side: Side, depth: u8) -> f32 {
    match side {
        // 我方的前排（depth 0）最靠近中間
        Side::Ally => (DEPTHS - 1 - depth) as f32 * ROW_DEPTH + ROW_DEPTH / 2.0,
        Side::Enemy => {
            DEPTHS as f32 * ROW_DEPTH + FRONT_GAP + depth as f32 * ROW_DEPTH + ROW_DEPTH / 2.0
        }
    }
}

/// 這一列中心的左右位置
fn row_v(row: u8) -> f32 {
    (row as f32 - (ROWS as f32 - 1.0) / 2.0) * lane_w()
}

/// 格子中心（地面）的畫面座標
pub fn cell_center(side: Side, pos: Pos) -> Vec2 {
    to_screen(row_v(pos.row), depth_u(side, pos.depth))
}

/// 格子在地面上的四個角（平行四邊形），往內縮 `inset`
fn cell_corners(side: Side, pos: Pos, inset: f32) -> [Vec2; 4] {
    let (v, u) = (row_v(pos.row), depth_u(side, pos.depth));
    let (hv, hu) = (lane_w() / 2.0 - inset, ROW_DEPTH / 2.0 - inset);
    [
        to_screen(v - hv, u - hu),
        to_screen(v + hv, u - hu),
        to_screen(v + hv, u + hu),
        to_screen(v - hv, u + hu),
    ]
}

fn fill_quad(c: [Vec2; 4], color: Color) {
    draw_triangle(c[0], c[1], c[2], color);
    draw_triangle(c[0], c[2], c[3], color);
}

fn quad_lines(c: [Vec2; 4], thickness: f32, color: Color) {
    for i in 0..4 {
        let (a, b) = (c[i], c[(i + 1) % 4]);
        draw_line(a.x, a.y, b.x, b.y, thickness, color);
    }
}

/// 在地面上標出一格（擺位畫面的開放格子）
pub fn draw_slot(side: Side, pos: Pos, fill: Color, border: Color) {
    let c = cell_corners(side, pos, 3.0);
    fill_quad(c, fill);
    quad_lines(c, 1.5, border);
}

/// 精靈立牌的範圍：站在格子中心
pub fn unit_rect(side: Side, pos: Pos) -> Rect {
    let c = cell_center(side, pos);
    let w = lane_w() + UNIT_EXTRA_W;
    Rect::new(c.x - w / 2.0, c.y + UNIT_FOOT - UNIT_H, w, UNIT_H)
}

fn tile_rect(u: &Unit) -> Rect {
    unit_rect(u.side, u.pos)
}

/// 由後往前的繪圖順序（畫面上越上面越先畫），前面的精靈會蓋住後面的
pub fn draw_order(units: &[Unit]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..units.len()).collect();
    order.sort_by(|&a, &b| {
        let ya = cell_center(units[a].side, units[a].pos).y;
        let yb = cell_center(units[b].side, units[b].pos).y;
        ya.total_cmp(&yb)
    });
    order
}

/// 畫面座標落在哪一格：先找立牌（前面的優先），沒有再看地面的格子
pub fn cell_at(side: Side, point: Vec2) -> Option<Pos> {
    let all = (0..ROWS).flat_map(|row| (0..DEPTHS).map(move |depth| Pos { row, depth }));
    let on_unit = all
        .clone()
        .filter(|&pos| unit_rect(side, pos).contains(point))
        .max_by(|&a, &b| {
            let ya = cell_center(side, a).y;
            let yb = cell_center(side, b).y;
            ya.total_cmp(&yb)
        });
    on_unit.or_else(|| {
        let (v, u) = to_field(point);
        all.into_iter().find(|&pos| {
            (v - row_v(pos.row)).abs() <= lane_w() / 2.0
                && (u - depth_u(side, pos.depth)).abs() <= ROW_DEPTH / 2.0
        })
    })
}

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

        for i in draw_order(&b.units) {
            draw_unit(&b.units[i], None);
            // 被打的精靈閃一下
            if damage_totals.iter().any(|d| d.0 == i) {
                gfx::rounded_rect(
                    tile_rect(&b.units[i]),
                    8.0,
                    gfx::with_alpha(palette::DAMAGE, 0.35 * (1.0 - t)),
                );
            }
        }

        // 攻擊線畫在精靈上面，斜向的戰場才看得到
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
                gfx::with_alpha(color, 0.9 - t * 0.6),
            );
        }

        for (target, damage, skill) in damage_totals {
            let r = tile_rect(&b.units[target]);
            let label = if skill {
                format!("-{damage}!")
            } else {
                format!("-{damage}")
            };
            // 傷害數字在立牌右上方，往上飄
            draw_tag(
                &label,
                vec2(r.x + r.w - 18.0, r.y - 4.0 - t * 10.0),
                palette::DAMAGE,
            );
        }
        for (target, amount) in heal_totals {
            let r = tile_rect(&b.units[target]);
            draw_tag(
                &format!("+{amount}"),
                vec2(r.center().x, r.y + r.h - t * 6.0),
                palette::HEAL,
            );
        }
        for caster in casters {
            let r = tile_rect(&b.units[caster]);
            draw_tag(
                &format!("「{}」", b.units[caster].skill.name),
                vec2(r.center().x - 8.0, r.y),
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

/// 雙方棋盤：底色、格線、「我方／敵方」、中間的前線
pub fn draw_boards() {
    let half = lane_w() * ROWS as f32 / 2.0;
    let board_len = ROW_DEPTH * DEPTHS as f32;
    for (side, start, tint) in [
        (Side::Ally, 0.0, palette::BOARD_ALLY),
        (Side::Enemy, board_len + FRONT_GAP, palette::BOARD_ENEMY),
    ] {
        fill_quad(
            [
                to_screen(-half, start),
                to_screen(half, start),
                to_screen(half, start + board_len),
                to_screen(-half, start + board_len),
            ],
            tint,
        );
        for row in 0..ROWS {
            for depth in 0..DEPTHS {
                quad_lines(
                    cell_corners(side, Pos { row, depth }, 2.0),
                    1.0,
                    palette::GRID,
                );
            }
        }
    }
    // 空出來的兩個角落放「敵方」「我方」
    let top = FIELD_BOTTOM + FIELD_LEN * forward().y;
    text::draw(
        "敵方",
        FIELD_MARGIN,
        top + 24.0,
        24.0,
        gfx::with_alpha(palette::ENEMY, 0.7),
    );
    text::right(
        "我方",
        W - FIELD_MARGIN,
        FIELD_BOTTOM - 6.0,
        24.0,
        gfx::with_alpha(palette::ALLY, 0.7),
    );
    // 前線：虛線
    let u = board_len + FRONT_GAP / 2.0;
    let mut v = -half;
    while v < half {
        let a = to_screen(v, u);
        let b = to_screen((v + 8.0).min(half), u);
        draw_line(a.x, a.y, b.x, b.y, 1.0, palette::TEXT_MUTED);
        v += 14.0;
    }
}

/// 畫一隻精靈的立牌。`footer` 會畫在左下角並取代血量數字（擺位畫面用來顯示陣型加成）
pub fn draw_unit(u: &Unit, footer: Option<&str>) {
    let r = tile_rect(u);
    // 腳下的影子
    let c = cell_center(u.side, u.pos);
    draw_ellipse(
        c.x,
        c.y + UNIT_FOOT - 2.0,
        r.w * 0.42,
        7.0,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.35),
    );
    if !u.alive() {
        gfx::rounded_rect(r, 8.0, gfx::with_alpha(palette::DEFEATED, 0.6));
        text::draw(&u.name, r.x + 8.0, r.y + 26.0, 24.0, palette::TEXT_MUTED);
        return;
    }
    let side_color = match u.side {
        Side::Ally => palette::ALLY,
        Side::Enemy => palette::ENEMY,
    };
    gfx::rounded_rect_gradient(
        r,
        8.0,
        gfx::shade(side_color, -0.5),
        gfx::shade(side_color, -0.72),
    );
    gfx::rounded_rect_lines(r, 8.0, 2.0, gfx::with_alpha(side_color, 0.9));
    text::draw(&u.name, r.x + 8.0, r.y + 26.0, 24.0, palette::TEXT);

    let ratio = u.hp as f32 / u.stats.hp as f32;
    let bar = Rect::new(r.x + 7.0, r.y + 31.0, r.w - 14.0, 6.0);
    gfx::rounded_rect(bar, 3.0, palette::HP_BAR_BG);
    if ratio > 0.0 {
        let fill = if ratio < 0.3 {
            palette::HP_BAR_LOW
        } else {
            palette::HP_BAR
        };
        gfx::rounded_rect(
            Rect::new(bar.x, bar.y, (bar.w * ratio).max(bar.h), bar.h),
            3.0,
            fill,
        );
    }
    match footer {
        Some(label) => text::draw(label, r.x + 7.0, r.y + r.h - 1.0, 12.0, palette::HIGHLIGHT),
        None => text::right(
            &format!("{}/{}", u.hp, u.stats.hp),
            r.x + r.w - 7.0,
            r.y + r.h - 1.0,
            12.0,
            palette::TEXT_DIM,
        ),
    }
}

/// 兩隻精靈之間的連線（攻擊線、預覽線），從立牌中心連到目標，目標那端畫一個圓點
pub fn draw_link(from: &Unit, to: &Unit, thickness: f32, color: Color) {
    let a = tile_rect(from).center();
    let b = tile_rect(to).center();
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
    }
}
