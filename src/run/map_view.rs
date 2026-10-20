//! 地圖畫面：左到右是各階段，點選可以走的節點。

use super::Run;
use super::map::{NodeKind, STAGES};
use super::ui;
use macroquad::prelude::*;

const LEFT: f32 = 60.0;
const RIGHT: f32 = 740.0;
const MIDDLE: f32 = 300.0;
const LANE_GAP: f32 = 110.0;
const RADIUS: f32 = 22.0;

fn node_pos(run: &Run, id: usize) -> Vec2 {
    let n = &run.map.nodes[id];
    let width = run.map.stages[n.stage].len() as f32;
    vec2(
        LEFT + n.stage as f32 * (RIGHT - LEFT) / (STAGES - 1) as f32,
        MIDDLE + (n.lane as f32 - (width - 1.0) / 2.0) * LANE_GAP,
    )
}

fn label(kind: NodeKind) -> (&'static str, Color) {
    match kind {
        NodeKind::Wild => ("W", Color::new(0.5, 0.75, 0.5, 1.0)),
        NodeKind::Elite => ("E", ORANGE),
        NodeKind::Shop => ("$", GOLD),
        NodeKind::Rest => ("R", SKYBLUE),
        NodeKind::Event => ("?", VIOLET),
        NodeKind::Boss => ("B", RED),
    }
}

/// 畫出地圖；玩家點了可以走的節點時回傳它
pub fn frame(run: &Run) -> Option<usize> {
    draw_text("Choose your path", 40.0, 50.0, 36.0, WHITE);

    for (id, n) in run.map.nodes.iter().enumerate() {
        let a = node_pos(run, id);
        for &t in &n.next {
            let b = node_pos(run, t);
            let walked = run.path.windows(2).any(|w| w == [id, t]);
            let (thick, color) = if walked {
                (4.0, WHITE)
            } else {
                (2.0, DARKGRAY)
            };
            draw_line(a.x, a.y, b.x, b.y, thick, color);
        }
    }

    let choices = run.choices();
    let mut picked = None;
    for (id, n) in run.map.nodes.iter().enumerate() {
        let p = node_pos(run, id);
        let (text, color) = label(n.kind);
        let can_go = choices.contains(&id);
        let hover = can_go && p.distance(ui::mouse()) <= RADIUS;
        let fill = if run.path.contains(&id) || can_go {
            color
        } else {
            Color::new(color.r * 0.4, color.g * 0.4, color.b * 0.4, 1.0)
        };
        draw_circle(p.x, p.y, RADIUS, fill);
        if can_go {
            let ring = if hover { WHITE } else { YELLOW };
            draw_circle_lines(p.x, p.y, RADIUS + 4.0, 3.0, ring);
        }
        if run.current() == Some(id) {
            draw_circle_lines(p.x, p.y, RADIUS + 4.0, 3.0, WHITE);
        }
        ui::centered_text(text, p, 28.0, BLACK);
        if hover && ui::clicked() {
            picked = Some(id);
        }
    }

    draw_text(
        "W wild   E elite   $ shop   R rest   ? event   B boss",
        40.0,
        570.0,
        24.0,
        LIGHTGRAY,
    );
    picked
}
