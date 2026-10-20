//! 整個遊戲的畫面切換：地圖 → 擺位 → 戰鬥 → 回到地圖，直到打倒頭目或戰敗。

use super::map::NodeKind;
use super::placement::PlacementScreen;
use super::{Reward, Run, catch_view, map_view, shop_view, ui};
use crate::battle::Outcome;
use crate::battle::view::BattleScreen;
use crate::text;
use macroquad::prelude::*;

enum Scene {
    Map,
    Placement(PlacementScreen),
    /// 戰鬥畫面，打贏後記下得到的獎勵（只發一次）
    Battle(BattleScreen, Option<Reward>),
    Shop,
    /// 捕捉節點出現的精靈
    Catch(Vec<&'static str>),
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
                let battle = self.run.start_battle(allies, enemies);
                Scene::Battle(BattleScreen::new(battle), None)
            }),
            Scene::Battle(screen, reward) => {
                screen.update(dt);
                screen.draw();
                let kind = self.run.current_kind();
                if screen.battle.outcome == Some(Outcome::Win)
                    && reward.is_none()
                    && let Some(kind) = kind
                {
                    *reward = Some(self.run.win_reward(kind));
                }
                if let Some(r) = reward {
                    let mut label = format!("獲得 {} 金幣", r.gold);
                    if r.balls > 0 {
                        label += &format!("、捕捉球 ×{}", r.balls);
                    }
                    text::centered(&label, vec2(400.0, 522.0), 24.0, GOLD);
                }
                screen.battle.outcome.and_then(|outcome| {
                    text::centered("點擊繼續", vec2(400.0, 562.0), 24.0, LIGHTGRAY);
                    ui::clicked().then(|| match outcome {
                        Outcome::Lose => Scene::GameOver,
                        Outcome::Win if kind == Some(NodeKind::Boss) => Scene::Cleared,
                        Outcome::Win => Scene::Map,
                    })
                })
            }
            Scene::Shop => shop_view::frame(&mut self.run).then_some(Scene::Map),
            Scene::Catch(offers) => catch_view::frame(&self.run, offers).map(|choice| {
                if let catch_view::Choice::Catch(name) = choice {
                    self.run.catch(name);
                }
                Scene::Map
            }),
            Scene::Notice(title) => {
                text::centered(title, center, 36.0, WHITE);
                text::centered("（尚未實作）", center + vec2(0.0, 40.0), 24.0, GRAY);
                text::centered("點擊繼續", center + vec2(0.0, 120.0), 24.0, LIGHTGRAY);
                ui::clicked().then_some(Scene::Map)
            }
            Scene::GameOver | Scene::Cleared => {
                let (title, color) = match self.scene {
                    Scene::GameOver => ("冒險失敗", RED),
                    _ => ("過關！", GREEN),
                };
                text::centered(title, center, 48.0, color);
                text::centered(
                    "點擊開始新的冒險",
                    center + vec2(0.0, 80.0),
                    24.0,
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
            NodeKind::Catch => Scene::Catch(self.run.catch_offers()),
            NodeKind::Shop => Scene::Shop,
            NodeKind::Rest => Scene::Notice("休息"),
            NodeKind::Event => Scene::Notice("事件"),
        }
    }
}
