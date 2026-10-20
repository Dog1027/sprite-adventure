//! 成長：在休息節點強化精靈。規則見 docs/DESIGN.md「精靈」。

use super::{Sprite, Stats};

/// 每次強化提升目前數值的百分比；暫定
pub const ENHANCE_PERCENT: u32 = 10;

/// 休息時玩家可以選的強化項目（節奏數值不能強化）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boost {
    Hp,
    Attack,
    Defense,
}

impl Boost {
    pub const ALL: [Boost; 3] = [Boost::Hp, Boost::Attack, Boost::Defense];

    pub fn label(self) -> &'static str {
        match self {
            Boost::Hp => "HP",
            Boost::Attack => "攻擊",
            Boost::Defense => "防禦",
        }
    }

    /// 這一項目前的數值
    pub fn value(self, stats: &Stats) -> u32 {
        match self {
            Boost::Hp => stats.hp,
            Boost::Attack => stats.attack,
            Boost::Defense => stats.defense,
        }
    }

    fn value_mut(self, stats: &mut Stats) -> &mut u32 {
        match self {
            Boost::Hp => &mut stats.hp,
            Boost::Attack => &mut stats.attack,
            Boost::Defense => &mut stats.defense,
        }
    }
}

impl Sprite {
    /// 選這一項會提升多少：目前數值 × ENHANCE_PERCENT%，四捨五入，至少 1。給畫面預覽用
    pub fn enhance_amount(&self, boost: Boost) -> u32 {
        let v = boost.value(&self.stats);
        ((v * ENHANCE_PERCENT + 50) / 100).max(1)
    }

    /// 強化一項能力值，回傳提升了多少
    pub fn enhance(&mut self, boost: Boost) -> u32 {
        let amount = self.enhance_amount(boost);
        *boost.value_mut(&mut self.stats) += amount;
        amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sprite(hp: u32, attack: u32, defense: u32) -> Sprite {
        let stats = Stats {
            hp,
            attack,
            defense,
            interval: 2,
            start_turn: 3,
            skill_every: 4,
        };
        Sprite::new("s", stats)
    }

    #[test]
    fn enhance_adds_percent_of_current_value() {
        let mut s = sprite(120, 30, 20);
        assert_eq!(s.enhance(Boost::Hp), 12);
        assert_eq!(s.stats.hp, 132);
        assert_eq!(s.enhance(Boost::Attack), 3);
        assert_eq!(s.stats.attack, 33);
    }

    #[test]
    fn enhance_rounds_and_gives_at_least_one() {
        let mut s = sprite(100, 10, 0);
        assert_eq!(s.enhance(Boost::Defense), 1);
        assert_eq!(s.stats.defense, 1);
        // 防禦 5 → 0.5 四捨五入為 1
        assert_eq!(sprite(100, 10, 5).enhance_amount(Boost::Defense), 1);
    }

    #[test]
    fn enhance_keeps_rhythm() {
        let mut s = sprite(100, 10, 10);
        for b in Boost::ALL {
            s.enhance(b);
        }
        let st = s.stats;
        assert_eq!((st.interval, st.start_turn, st.skill_every), (2, 3, 4));
    }
}
