//! 地圖畫面：由下往上是各階段，最上面是頭目。點選可以走的節點。

use super::Run;
use super::map::{NodeKind, STAGES};
use super::ui::{self, HEADER_H, MARGIN};
use crate::gfx;
use crate::palette;
use crate::screen::{H, W};
use crate::text;
use macroquad::prelude::*;

/// 第一個階段與頭目的高度
const BOTTOM: f32 = 676.0;
const TOP: f32 = HEADER_H + 52.0;
const LANE_GAP: f32 = 96.0;
const RADIUS: f32 = 24.0;
/// 下方圖例的上緣
const LEGEND_TOP: f32 = H - 82.0;

fn node_pos(run: &Run, id: usize) -> Vec2 {
    let n = &run.map.nodes[id];
    let width = run.map.stages[n.stage].len() as f32;
    vec2(
        W / 2.0 + (n.lane as f32 - (width - 1.0) / 2.0) * LANE_GAP,
        BOTTOM - n.stage as f32 * (BOTTOM - TOP) / (STAGES - 1) as f32,
    )
}

fn label(kind: NodeKind) -> (&'static str, &'static str, Color) {
    match kind {
        NodeKind::Wild => ("野", "野生", palette::NODE_WILD),
        NodeKind::Elite => ("菁", "菁英", palette::NODE_ELITE),
        NodeKind::Shop => ("商", "商店", palette::NODE_SHOP),
        NodeKind::Catch => ("捕", "捕捉", palette::NODE_CATCH),
        NodeKind::Rest => ("休", "休息", palette::NODE_REST),
        NodeKind::Event => ("事", "事件", palette::NODE_EVENT),
        NodeKind::Boss => ("王", "頭目", palette::NODE_BOSS),
    }
}

/// 畫出地圖；玩家點了可以走的節點時回傳它
pub fn frame(run: &Run) -> Option<usize> {
    ui::header("冒險地圖", Some(run));

    for (id, n) in run.map.nodes.iter().enumerate() {
        let a = node_pos(run, id);
        for &t in &n.next {
            let b = node_pos(run, t);
            let walked = run.path.windows(2).any(|w| w == [id, t]);
            let (thick, color) = if walked {
                (4.0, palette::PATH_TAKEN)
            } else {
                (2.0, palette::PATH)
            };
            draw_line(a.x, a.y, b.x, b.y, thick, color);
        }
    }

    let choices = run.choices();
    // 可以走的節點外圈會呼吸，提示玩家點這裡
    let pulse = (get_time() as f32 * 4.0).sin() * 0.5 + 0.5;
    let mut picked = None;
    for (id, n) in run.map.nodes.iter().enumerate() {
        let p = node_pos(run, id);
        let (ch, _, color) = label(n.kind);
        let can_go = choices.contains(&id);
        let visited = run.path.contains(&id);
        let radius = if n.kind == NodeKind::Boss {
            RADIUS + 6.0
        } else {
            RADIUS
        };
        let fill = if visited || can_go {
            color
        } else {
            gfx::shade(color, -0.35)
        };
        if can_go {
            draw_circle(
                p.x,
                p.y,
                radius + 6.0 + pulse * 3.0,
                gfx::with_alpha(palette::NODE_RING, 0.18),
            );
            draw_circle_lines(p.x, p.y, radius + 5.0, 3.0, palette::NODE_RING);
        }
        draw_circle(p.x, p.y + 3.0, radius, Color::new(0.0, 0.0, 0.0, 0.35));
        draw_circle(p.x, p.y, radius, fill);
        // 上半部亮一點，有立體感
        draw_circle(
            p.x,
            p.y - radius * 0.25,
            radius * 0.7,
            gfx::with_alpha(gfx::shade(fill, 0.25), 0.5),
        );
        if run.current() == Some(id) {
            draw_circle_lines(p.x, p.y, radius + 5.0, 3.0, palette::NODE_FOCUS);
        }
        let text_color = if visited || can_go {
            palette::TEXT_ON_LIGHT
        } else {
            gfx::with_alpha(palette::TEXT_ON_LIGHT, 0.85)
        };
        text::centered(ch, p, 24.0, text_color);
        // 手指比較粗，點的範圍比圓大一點
        if can_go && ui::clicked() && p.distance(ui::mouse()) <= radius + 12.0 {
            picked = Some(id);
        }
    }

    draw_legend();
    picked
}

/// 下方的圖例：每種節點的顏色與名稱
fn draw_legend() {
    let kinds = [
        NodeKind::Wild,
        NodeKind::Elite,
        NodeKind::Catch,
        NodeKind::Shop,
        NodeKind::Rest,
        NodeKind::Event,
        NodeKind::Boss,
    ];
    let panel = Rect::new(MARGIN, LEGEND_TOP, W - 2.0 * MARGIN, 66.0);
    gfx::rounded_rect(panel, 10.0, palette::CHIP);
    let per_row = 4;
    let col_w = panel.w / per_row as f32;
    for (i, &kind) in kinds.iter().enumerate() {
        let (ch, name, color) = label(kind);
        let x = panel.x + 14.0 + (i % per_row) as f32 * col_w;
        let y = panel.y + 18.0 + (i / per_row) as f32 * 30.0;
        draw_circle(x + 9.0, y, 11.0, color);
        text::centered(ch, vec2(x + 9.0, y), 12.0, palette::TEXT_ON_LIGHT);
        text::draw(name, x + 26.0, y + 7.0, 18.0, palette::TEXT_DIM);
    }
}
