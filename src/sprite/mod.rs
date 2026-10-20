//! 精靈：能力值、技能、精靈資料、之後的成長與捕捉。規則見 docs/DESIGN.md「精靈」「技能」「捕捉」。
//!
//! 這個模組不依賴其他遊戲模組；戰鬥與流程都透過 `Sprite`、`Stats`、`Skill`（`skill::Range`）使用它。

pub mod skill;
pub mod species;

pub use skill::Skill;

/// 能力值與節奏數值，戰鬥只讀取這些值
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stats {
    pub hp: u32,
    pub attack: u32,
    pub defense: u32,
    /// 每幾回合攻擊一次
    pub interval: u32,
    /// 第幾回合開始攻擊（從 1 起算）
    pub start_turn: u32,
    /// 每幾次攻擊有一次改用技能
    pub skill_every: u32,
}

/// 玩家隊伍中或敵方的一隻精靈
#[derive(Clone, Debug)]
pub struct Sprite {
    pub name: String,
    pub stats: Stats,
    /// 每隻精靈一種技能
    #[allow(dead_code)] // 技能效果由戰鬥模組執行，尚未實作
    pub skill: Skill,
}

impl Sprite {
    /// 建立精靈，技能為預設的普攻；要指定技能用 `with_skill`
    pub fn new(name: &str, stats: Stats) -> Self {
        assert!(stats.interval >= 1 && stats.start_turn >= 1 && stats.skill_every >= 1);
        Self {
            name: name.to_owned(),
            stats,
            skill: Skill::basic(),
        }
    }

    pub fn with_skill(mut self, skill: Skill) -> Self {
        self.skill = skill;
        self
    }
}
