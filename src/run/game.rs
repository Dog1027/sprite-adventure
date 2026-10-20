//! 整個遊戲的畫面切換：地圖 → 擺位 → 戰鬥 → 回到地圖，直到打倒頭目或戰敗。

use super::map::NodeKind;
use super::placement::PlacementScreen;
use super::{Run, map_view, ui};
use crate::battle::Outcome;
use crate::battle::view::BattleScreen;
use macroquad::prelude::*;

enum Scene {
    Map,
    Placement(PlacementScreen),
    Battle(BattleScreen),
    /// 尚未實作的節點
    Notice(&'static str),
    GameOver,
    Cleared,
}

pub struct Game {
    run: Run,
    scene: Scene,
    seed: u64,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        Self {
            run: Run::new(seed),
            scene: Scene::Map,
            seed,
        }
    }

    /// 每一幀呼叫一次：處理輸入、更新、畫出目前畫面
    pub fn frame(&mut self, dt: f32) {
        let center = vec2(screen_width() / 2.0, 260.0);
        let next = match &mut self.scene {
            Scene::Map => map_view::frame(&self.run).map(|node| self.enter(node)),
            Scene::Placement(screen) => screen.frame(&self.run).map(|allies| {
                let enemies = std::mem::take(&mut screen.enemies);
                Scene::Battle(BattleScreen::new(self.run.start_battle(allies, enemies)))
            }),
            Scene::Battle(screen) => {
                screen.update(dt);
                screen.draw();
                screen.battle.outcome.and_then(|outcome| {
                    draw_text("Click to continue", 300.0, 560.0, 26.0, LIGHTGRAY);
                    ui::clicked().then(|| match outcome {
                        Outcome::Lose => Scene::GameOver,
                        Outcome::Win if self.run.current_kind() == Some(NodeKind::Boss) => {
                            Scene::Cleared
                        }
                        Outcome::Win => Scene::Map,
                    })
                })
            }
            Scene::Notice(text) => {
                ui::centered_text(text, center, 36.0, WHITE);
                ui::centered_text(
                    "(not implemented yet)",
                    center + vec2(0.0, 40.0),
                    24.0,
                    GRAY,
                );
                ui::centered_text(
                    "Click to continue",
                    center + vec2(0.0, 120.0),
                    24.0,
                    LIGHTGRAY,
                );
                ui::clicked().then_some(Scene::Map)
            }
            Scene::GameOver | Scene::Cleared => {
                let (text, color) = match self.scene {
                    Scene::GameOver => ("Game Over", RED),
                    _ => ("Floor cleared!", GREEN),
                };
                ui::centered_text(text, center, 60.0, color);
                ui::centered_text(
                    "Click to start a new run",
                    center + vec2(0.0, 80.0),
                    26.0,
                    LIGHTGRAY,
                );
                if ui::clicked() {
                    self.seed += 1;
                    self.run = Run::new(self.seed);
                    Some(Scene::Map)
                } else {
                    None
                }
            }
        };
        if let Some(scene) = next {
            self.scene = scene;
        }
    }

    fn enter(&mut self, node: usize) -> Scene {
        self.run.enter(node);
        match self.run.map.nodes[node].kind {
            kind @ (NodeKind::Wild | NodeKind::Elite | NodeKind::Boss) => {
                let enemies = self.run.encounter(kind);
                Scene::Placement(PlacementScreen::new(&self.run, enemies))
            }
            NodeKind::Shop => Scene::Notice("Shop"),
            NodeKind::Rest => Scene::Notice("Rest"),
            NodeKind::Event => Scene::Notice("Event"),
        }
    }
}
