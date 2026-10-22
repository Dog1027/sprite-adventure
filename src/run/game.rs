//! 整個遊戲的畫面切換：地圖 → 擺位 → 戰鬥 → 回到地圖，直到打倒頭目或戰敗。

#[cfg(not(target_arch = "wasm32"))]
pub mod shots;

use super::map::NodeKind;
use super::placement::PlacementScreen;
use super::rest_view::RestScreen;
use super::ui::{self, BOTTOM_BUTTON, HEADER_H, MARGIN, Style};
use super::{Reward, Run, catch_view, map_view, shop_view};
use crate::art;
use crate::battle::Outcome;
use crate::battle::view::{BOARD_BOTTOM, BattleScreen};
use crate::gfx;
use crate::palette;
use crate::screen::{H, W};
use crate::text;
use macroquad::prelude::*;

/// 換畫面時淡入的秒數
const FADE_SECONDS: f32 = 0.25;

enum Scene {
    Map,
    Placement(PlacementScreen),
    /// 戰鬥畫面，打贏後記下得到的獎勵（只發一次）
    Battle(BattleScreen, Option<Reward>),
    Shop,
    Rest(RestScreen),
    /// 捕捉節點出現的精靈
    Catch(Vec<&'static str>),
    /// 尚未實作的節點
    Notice(&'static str),
    GameOver,
    Cleared,
}

impl Scene {
    /// 這個畫面的場景背景
    fn backdrop(&self) -> art::Scene {
        match self {
            Scene::Map => art::Scene::Map,
            Scene::Placement(_) | Scene::Battle(..) => art::Scene::Battle,
            Scene::Shop => art::Scene::Shop,
            Scene::Rest(_) => art::Scene::Rest,
            Scene::Catch(_) => art::Scene::Catch,
            Scene::Notice(_) => art::Scene::Event,
            Scene::GameOver => art::Scene::Defeat,
            Scene::Cleared => art::Scene::Victory,
        }
    }
}

pub struct Game {
    run: Run,
    scene: Scene,
    seed: u64,
    /// 淡入的進度，1 為全黑、0 為完全顯示
    fade: f32,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        Self {
            run: Run::new(seed),
            scene: Scene::Map,
            seed,
            fade: 1.0,
        }
    }

    /// 每一幀呼叫一次：處理輸入、更新、畫出目前畫面
    pub fn frame(&mut self, dt: f32) {
        if !art::draw_scene(self.scene.backdrop()) {
            ui::background();
        }
        let next = match &mut self.scene {
            Scene::Map => map_view::frame(&self.run).map(|node| self.enter(node)),
            Scene::Placement(screen) => screen.frame(&self.run).map(|allies| {
                let enemies = std::mem::take(&mut screen.enemies);
                let battle = self.run.start_battle(allies, enemies);
                Scene::Battle(BattleScreen::new(battle), None)
            }),
            Scene::Battle(screen, reward) => battle_frame(&mut self.run, screen, reward, dt),
            Scene::Shop => shop_view::frame(&mut self.run).then_some(Scene::Map),
            Scene::Rest(screen) => screen.frame(&mut self.run).then_some(Scene::Map),
            Scene::Catch(offers) => catch_view::frame(&self.run, offers).map(|choice| {
                if let catch_view::Choice::Catch(name) = choice {
                    self.run.catch(name);
                }
                Scene::Map
            }),
            Scene::Notice(title) => {
                ui::header(title, Some(&self.run));
                let r = Rect::new(MARGIN, HEADER_H + 120.0, W - 2.0 * MARGIN, 140.0);
                ui::card(r);
                text::centered(title, vec2(W / 2.0, r.y + 56.0), 36.0, palette::TEXT);
                text::centered(
                    "尚未實作",
                    vec2(W / 2.0, r.y + 100.0),
                    24.0,
                    palette::TEXT_MUTED,
                );
                ui::button(BOTTOM_BUTTON, "繼續", Style::Primary, true).then_some(Scene::Map)
            }
            Scene::GameOver | Scene::Cleared => {
                let cleared = matches!(self.scene, Scene::Cleared);
                if end_frame(&self.run, cleared) {
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
            self.fade = 1.0;
        }

        if self.fade > 0.0 {
            draw_rectangle(
                0.0,
                0.0,
                W,
                H,
                gfx::with_alpha(palette::LETTERBOX, self.fade),
            );
            self.fade = (self.fade - dt / FADE_SECONDS).max(0.0);
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
            NodeKind::Rest => Scene::Rest(RestScreen::default()),
            NodeKind::Event => Scene::Notice("事件"),
        }
    }
}

/// 戰鬥畫面：上方回合數，下方是速度按鈕，分出勝負後換成結果與獎勵
fn battle_frame(
    run: &mut Run,
    screen: &mut BattleScreen,
    reward: &mut Option<Reward>,
    dt: f32,
) -> Option<Scene> {
    screen.update(dt);
    ui::header(&format!("第 {} 回合", screen.battle.turn), Some(run));
    let kind = run.current_kind();
    let title = match kind {
        Some(NodeKind::Elite) => "菁英戰",
        Some(NodeKind::Boss) => "頭目戰",
        _ => "野生戰鬥",
    };
    text::centered(
        title,
        vec2(W / 2.0, HEADER_H + 30.0),
        24.0,
        palette::TEXT_DIM,
    );
    screen.draw();

    let Some(outcome) = screen.battle.outcome else {
        let gap = 10.0;
        let w = (BOTTOM_BUTTON.w - gap) / 2.0;
        let fast = Rect::new(BOTTOM_BUTTON.x, BOTTOM_BUTTON.y, w, BOTTOM_BUTTON.h);
        let skip = Rect::new(fast.x + w + gap, fast.y, w, fast.h);
        let style = if screen.fast {
            Style::Selected
        } else {
            Style::Normal
        };
        if ui::button(fast, "快轉", style, true) {
            screen.fast = !screen.fast;
        }
        if ui::button(skip, "跳過", Style::Normal, true) {
            screen.skip();
        }
        text::centered(
            "戰鬥自動進行",
            vec2(W / 2.0, BOARD_BOTTOM + 40.0),
            18.0,
            palette::TEXT_MUTED,
        );
        return None;
    };

    if outcome == Outcome::Win
        && reward.is_none()
        && let Some(kind) = kind
    {
        *reward = Some(run.win_reward(kind));
    }
    let r = Rect::new(MARGIN, BOARD_BOTTOM + 12.0, W - 2.0 * MARGIN, 92.0);
    ui::card(r);
    let (title, color) = match outcome {
        Outcome::Win => ("勝利", palette::SUCCESS),
        Outcome::Lose => ("戰敗", palette::FAILURE),
    };
    text::draw(title, r.x + 20.0, r.y + 58.0, 36.0, color);
    if let Some(reward) = reward {
        // 獎勵：金幣，有掉落時再加捕捉球
        let mut x = r.x + r.w - 20.0;
        let y = r.center().y;
        let mut item = |icon: ui::Icon, value: String| {
            let label = format!("+{value}");
            let w = text::measure(&label, 24.0).width;
            text::draw(&label, x - w, y + 9.0, 24.0, palette::HIGHLIGHT);
            icon(vec2(x - w - 16.0, y), 10.0);
            x -= w + 44.0;
        };
        if reward.balls > 0 {
            item(ui::ball_icon, reward.balls.to_string());
        }
        item(ui::coin_icon, reward.gold.to_string());
    }
    let label = match outcome {
        Outcome::Win => "繼續",
        Outcome::Lose => "查看結果",
    };
    ui::button(BOTTOM_BUTTON, label, Style::Primary, true).then(|| match outcome {
        Outcome::Lose => Scene::GameOver,
        Outcome::Win if kind == Some(NodeKind::Boss) => Scene::Cleared,
        Outcome::Win => Scene::Map,
    })
}

/// 冒險結束（過關或失敗）；按下再來一局時回傳 true
fn end_frame(run: &Run, cleared: bool) -> bool {
    let (title, color) = if cleared {
        ("過關！", palette::SUCCESS)
    } else {
        ("冒險失敗", palette::FAILURE)
    };
    let r = Rect::new(MARGIN, 200.0, W - 2.0 * MARGIN, 220.0);
    ui::card(r);
    text::centered(title, vec2(W / 2.0, r.y + 70.0), 48.0, color);
    let lines = [
        format!("走過 {} 個節點", run.path.len()),
        format!("隊伍 {} 隻精靈", run.party.len()),
        format!("剩下 {} 金幣、{} 顆捕捉球", run.gold, run.balls),
    ];
    for (i, line) in lines.iter().enumerate() {
        text::centered(
            line,
            vec2(W / 2.0, r.y + 128.0 + i as f32 * 28.0),
            18.0,
            palette::TEXT_DIM,
        );
    }
    ui::button(BOTTOM_BUTTON, "再來一局", Style::Primary, true)
}
