//! 流程：一局冒險的狀態、分支地圖、節點（戰鬥、商店、休息…）。規則見 docs/DESIGN.md「一局的流程」。
//!
//! 流程決定下一場戰鬥的陣容與位置，交給戰鬥模組執行，再依 `Outcome` 決定接下來的發展。

pub mod map;

use crate::battle::formation::{self, Formation};
use crate::battle::{Battle, Side, Unit};
use crate::rng::Rng;
use crate::sprite::{Sprite, species};
use map::{Map, NodeKind};

/// 開局隊伍；暫定，之後由捕捉決定
const STARTERS: [&str; 4] = ["A1", "A2", "A3", "A4"];
/// 敵方會出現的精靈；暫定
const ENEMY_POOL: [&str; 3] = ["E1", "E2", "E3"];

/// 一局冒險的狀態
pub struct Run {
    pub party: Vec<Sprite>,
    /// 已解鎖的陣型
    pub formations: Vec<Formation>,
    pub map: Map,
    /// 目前所在的節點；還沒出發時為 `None`
    pub current: Option<usize>,
    rng: Rng,
}

impl Run {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        Self {
            party: STARTERS.map(species::create).to_vec(),
            // 暫定：先開放全部範例陣型，之後改為開局只有基本陣型
            formations: formation::samples(),
            map: Map::generate(&mut rng),
            current: None,
            rng,
        }
    }

    /// 下一步可以走的節點
    pub fn choices(&self) -> Vec<usize> {
        match self.current {
            None => self.map.stages[0].clone(),
            Some(id) => self.map.nodes[id].next.clone(),
        }
    }

    pub fn enter(&mut self, node: usize) {
        assert!(self.choices().contains(&node), "節點 {node} 不能走");
        self.current = Some(node);
    }

    pub fn current_kind(&self) -> Option<NodeKind> {
        self.current.map(|id| self.map.nodes[id].kind)
    }

    /// 產生這個節點的敵方隊伍（含陣型與位置），戰前就給玩家看
    pub fn encounter(&mut self, kind: NodeKind) -> Vec<Unit> {
        let formations = formation::samples();
        let f = &formations[self.rng.below(formations.len())];
        let slots = f.slots.len();
        // 上場隻數；暫定
        let count = match kind {
            NodeKind::Wild => formation::MIN_UNITS,
            NodeKind::Elite => slots.min(formation::MIN_UNITS + 1 + self.rng.below(2)),
            _ => slots,
        };

        // 隨機挑格子，再隨機挑精靈
        let mut open: Vec<usize> = (0..slots).collect();
        let mut chosen = vec![false; slots];
        for _ in 0..count {
            chosen[open.remove(self.rng.below(open.len()))] = true;
        }
        let sprites: Vec<Option<Sprite>> = chosen
            .iter()
            .map(|&c| c.then(|| species::create(ENEMY_POOL[self.rng.below(ENEMY_POOL.len())])))
            .collect();
        let placement: Vec<Option<&Sprite>> = sprites.iter().map(Option::as_ref).collect();
        f.place(&placement, Side::Enemy).expect("敵方至少 3 隻")
    }

    pub fn start_battle(&mut self, mut allies: Vec<Unit>, enemies: Vec<Unit>) -> Battle {
        allies.extend(enemies);
        let seed = self.rng.below(usize::MAX) as u64;
        Battle::new(allies, seed)
    }

    /// 暫時用來直接開戰：我方以第一個陣型擺上全部隊伍，打一場野生戰鬥
    pub fn next_battle(&mut self) -> Battle {
        let party: Vec<Option<&Sprite>> = self.party.iter().map(Some).collect();
        let allies = self.formations[0]
            .place(&party, Side::Ally)
            .expect("隊伍至少 3 隻");
        let enemies = self.encounter(NodeKind::Wild);
        self.start_battle(allies, enemies)
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
    fn encounter_sizes() {
        let mut run = Run::new(5);
        for _ in 0..50 {
            assert_eq!(run.encounter(NodeKind::Wild).len(), 3);
            let elite = run.encounter(NodeKind::Elite).len();
            assert!((3..=5).contains(&elite));
            let boss = run.encounter(NodeKind::Boss);
            assert!(boss.len() >= 3 && boss.iter().all(|u| u.side == Side::Enemy));
        }
    }
}
