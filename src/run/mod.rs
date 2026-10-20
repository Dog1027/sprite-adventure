//! 流程：一局冒險的狀態、分支地圖、節點（戰鬥、商店、休息…）。規則見 docs/DESIGN.md「一局的流程」。
//!
//! 流程決定下一場戰鬥的陣容與位置，交給戰鬥模組執行，再依 `Outcome` 決定接下來的發展。

use crate::battle::{Battle, Side, formation};
use crate::sprite::{Sprite, species};

/// 地圖上的節點種類
#[allow(dead_code)] // 地圖尚未實作
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Wild,
    Elite,
    Shop,
    Rest,
    Event,
    Boss,
}

/// 一局冒險的狀態
pub struct Run {
    pub party: Vec<Sprite>,
    pub seed: u64,
}

impl Run {
    pub fn new(seed: u64) -> Self {
        let party = ["A1", "A2", "A3", "A4"].map(species::create).to_vec();
        Self { party, seed }
    }

    /// 產生下一場戰鬥。暫時使用固定的陣型與敵人，之後由陣型畫面與地圖節點決定。
    pub fn next_battle(&mut self) -> Battle {
        let formations = formation::samples();
        let party: Vec<Option<&Sprite>> = self.party.iter().map(Some).collect();
        let mut units = formations[0]
            .place(&party, Side::Ally)
            .expect("隊伍至少 3 隻");

        let enemies = ["E1", "E2", "E3"].map(species::create);
        let enemies: Vec<Option<&Sprite>> = enemies.iter().map(Some).collect();
        units.extend(
            formations[2]
                .place(&enemies, Side::Enemy)
                .expect("敵人至少 3 隻"),
        );

        self.seed += 1;
        Battle::new(units, self.seed)
    }
}
