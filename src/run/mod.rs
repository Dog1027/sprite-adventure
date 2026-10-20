//! 流程：一局冒險的狀態、分支地圖、節點（戰鬥、商店、休息…）。規則見 docs/DESIGN.md「一局的流程」。
//!
//! 流程決定下一場戰鬥的陣容與位置，交給戰鬥模組執行，再依 `Outcome` 決定接下來的發展。

mod catch_view;
mod game;
pub mod map;
mod map_view;
mod placement;
mod shop_view;
mod ui;

pub use game::Game;

use crate::battle::formation::{self, Formation};
use crate::battle::{Battle, Side, Unit};
use crate::rng::Rng;
use crate::sprite::{Sprite, generate, species};
use map::{Map, NodeKind};

/// 捕捉節點一次出現幾隻（選一隻抓）；暫定
pub const CATCH_OFFERS: usize = 3;
/// 開局的捕捉球數量
const STARTING_BALLS: u32 = 3;
/// 商店的捕捉球價格（金幣）；暫定
pub const BALL_PRICE: u32 = 15;

/// 打贏戰鬥的獎勵
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reward {
    pub gold: u32,
    pub balls: u32,
}

/// 獎勵的金幣與掉落捕捉球的機率（%）；暫定
fn reward_table(kind: NodeKind) -> (u32, usize) {
    match kind {
        NodeKind::Elite => (25, 100),
        NodeKind::Boss => (50, 100),
        _ => (10, 30),
    }
}

/// 敵方上場隻數：野生從 1 隻開始隨階段增加；暫定
fn enemy_count(kind: NodeKind, stage: usize) -> usize {
    let wild = (1 + stage / 2).min(3);
    match kind {
        NodeKind::Elite => wild + 1,
        NodeKind::Boss => 4,
        _ => wild,
    }
}

/// 一局冒險的狀態
pub struct Run {
    /// 隊伍沒有上限
    pub party: Vec<Sprite>,
    /// 捕捉球，每次捕捉用掉一個；目前不影響捕捉率
    pub balls: u32,
    pub gold: u32,
    /// 已解鎖的陣型
    pub formations: Vec<Formation>,
    pub map: Map,
    /// 走過的節點，最後一個是目前所在的節點
    pub path: Vec<usize>,
    rng: Rng,
}

impl Run {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        Self {
            // 開局只有一隻精靈，之後在捕捉節點增加
            party: vec![species::starter()],
            balls: STARTING_BALLS,
            gold: 0,
            // 暫定：先開放全部範例陣型，之後改為開局只有基本陣型
            formations: formation::samples(),
            map: Map::generate(&mut rng),
            path: Vec::new(),
            rng,
        }
    }

    /// 下一步可以走的節點
    pub fn choices(&self) -> Vec<usize> {
        match self.current() {
            None => self.map.stages[0].clone(),
            Some(id) => self.map.nodes[id].next.clone(),
        }
    }

    pub fn enter(&mut self, node: usize) {
        assert!(self.choices().contains(&node), "節點 {node} 不能走");
        self.path.push(node);
    }

    pub fn current(&self) -> Option<usize> {
        self.path.last().copied()
    }

    pub fn current_kind(&self) -> Option<NodeKind> {
        self.current().map(|id| self.map.nodes[id].kind)
    }

    /// 產生這個節點的敵方隊伍（含陣型與位置），戰前就給玩家看
    pub fn encounter(&mut self, kind: NodeKind) -> Vec<Unit> {
        let formations = formation::samples();
        let f = &formations[self.rng.below(formations.len())];
        let slots = f.slots.len();
        let stage = self.current().map_or(0, |id| self.map.nodes[id].stage);
        let count = enemy_count(kind, stage).min(slots);

        // 隨機挑格子，再隨機挑精靈
        let mut open: Vec<usize> = (0..slots).collect();
        let mut chosen = vec![false; slots];
        for _ in 0..count {
            chosen[open.remove(self.rng.below(open.len()))] = true;
        }
        let sprites: Vec<Option<Sprite>> = chosen
            .iter()
            .map(|&c| c.then(|| generate::enemy(&mut self.rng, stage)))
            .collect();
        let placement: Vec<Option<&Sprite>> = sprites.iter().map(Option::as_ref).collect();
        f.place(&placement, Side::Enemy).expect("敵方至少 1 隻")
    }

    /// 捕捉節點出現的精靈（種類名稱，不重複）
    pub fn catch_offers(&mut self) -> Vec<&'static str> {
        let stage = self.current().map_or(0, |id| self.map.nodes[id].stage);
        generate::catch_offers(&mut self.rng, stage, CATCH_OFFERS)
    }

    /// 捕捉：用掉一個捕捉球，目前必定成功（見 DESIGN.md「捕捉」）。沒有球時回傳 false。
    pub fn catch(&mut self, name: &str) -> bool {
        if self.balls == 0 {
            return false;
        }
        self.balls -= 1;
        self.party.push(species::create(name));
        true
    }

    /// 打贏戰鬥：得到金幣，有機會得到捕捉球
    pub fn win_reward(&mut self, kind: NodeKind) -> Reward {
        let (gold, ball_chance) = reward_table(kind);
        let balls = u32::from(self.rng.below(100) < ball_chance);
        self.gold += gold;
        self.balls += balls;
        Reward { gold, balls }
    }

    /// 在商店買一個捕捉球；金幣不夠時回傳 false
    pub fn buy_ball(&mut self) -> bool {
        if self.gold < BALL_PRICE {
            return false;
        }
        self.gold -= BALL_PRICE;
        self.balls += 1;
        true
    }

    pub fn start_battle(&mut self, mut allies: Vec<Unit>, enemies: Vec<Unit>) -> Battle {
        allies.extend(enemies);
        let seed = self.rng.below(usize::MAX) as u64;
        Battle::new(allies, seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walks_from_first_stage_to_boss() {
        let mut run = Run::new(3);
        while let Some(&next) = run.choices().first() {
            run.enter(next);
        }
        assert_eq!(run.current_kind(), Some(NodeKind::Boss));
    }

    #[test]
    #[should_panic]
    fn cannot_skip_stages() {
        let mut run = Run::new(3);
        let boss = run.map.stages[map::STAGES - 1][0];
        run.enter(boss);
    }

    #[test]
    fn starts_with_one_sprite() {
        assert_eq!(Run::new(1).party.len(), 1);
    }

    #[test]
    fn enemy_count_grows_with_stage() {
        let wild: Vec<usize> = (0..7).map(|s| enemy_count(NodeKind::Wild, s)).collect();
        assert_eq!(wild, vec![1, 1, 2, 2, 3, 3, 3]);
        assert_eq!(enemy_count(NodeKind::Elite, 2), 3);
        assert_eq!(enemy_count(NodeKind::Boss, 7), 4);
    }

    #[test]
    fn first_battle_has_one_enemy() {
        let mut run = Run::new(5);
        let first = run.choices()[0];
        run.enter(first);
        for _ in 0..20 {
            assert_eq!(run.encounter(NodeKind::Wild).len(), 1);
        }
        let boss = run.encounter(NodeKind::Boss);
        assert!(!boss.is_empty() && boss.len() <= 4);
        assert!(boss.iter().all(|u| u.side == Side::Enemy));
    }

    #[test]
    fn each_catch_uses_a_ball() {
        let mut run = Run::new(2);
        let offers = run.catch_offers();
        assert_eq!(offers.len(), CATCH_OFFERS);
        assert!(
            offers
                .iter()
                .all(|o| offers.iter().filter(|x| *x == o).count() == 1)
        );
        assert_eq!(run.balls, STARTING_BALLS);
        for _ in 0..STARTING_BALLS {
            assert!(run.catch(offers[0]));
        }
        assert_eq!(run.party.len(), 1 + STARTING_BALLS as usize);
        assert_eq!(run.balls, 0);
        assert!(!run.catch(offers[0]));
        assert_eq!(run.party.len(), 1 + STARTING_BALLS as usize);
    }

    #[test]
    fn party_has_no_limit() {
        let mut run = Run::new(2);
        run.balls = 20;
        for _ in 0..20 {
            assert!(run.catch("ember_fox"));
        }
        assert_eq!(run.party.len(), 21);
    }

    #[test]
    fn winning_gives_gold_and_sometimes_balls() {
        let mut run = Run::new(3);
        let mut dropped = 0;
        for _ in 0..200 {
            let r = run.win_reward(NodeKind::Wild);
            assert_eq!(r.gold, 10);
            assert!(r.balls <= 1);
            dropped += r.balls;
        }
        assert!((30..90).contains(&dropped), "掉落 {dropped} 次，應約 30%");
        assert_eq!(run.gold, 2000);
        assert_eq!(run.balls, STARTING_BALLS + dropped);
        assert_eq!(run.win_reward(NodeKind::Elite).balls, 1);
    }

    #[test]
    fn buying_balls_costs_gold() {
        let mut run = Run::new(3);
        assert!(!run.buy_ball());
        run.gold = BALL_PRICE * 2 - 1;
        assert!(run.buy_ball());
        assert!(!run.buy_ball());
        assert_eq!(run.gold, BALL_PRICE - 1);
        assert_eq!(run.balls, STARTING_BALLS + 1);
    }
}
