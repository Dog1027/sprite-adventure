//! 戰鬥：棋盤、回合節奏、目標選擇、傷害與結算。規則見 docs/DESIGN.md「陣型」「戰鬥」。
//!
//! 對外介面：用 `Unit::new` 把精靈擺上棋盤，`Battle::new` 開始戰鬥，
//! 反覆呼叫 `Battle::step` 直到 `outcome` 有結果。畫面在 `view`。

mod damage;
pub mod formation;
mod targeting;

pub use targeting::preview;
pub mod view;

use damage::{heal_amount, skill_damage};

use crate::rng::Rng;
use crate::sprite::skill::Range;
use crate::sprite::{Skill, Sprite, Stats};

/// 每邊棋盤的深度（前後）與列數（上下）
pub const DEPTHS: u8 = 4;
pub const ROWS: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Ally,
    Enemy,
}

/// 棋盤位置。`depth` 0 為最前方，3 為最後方；`row` 0～2 為第 1～3 列。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos {
    pub row: u8,
    pub depth: u8,
}

/// 這一回合是否輪到攻擊
pub fn acts_on(stats: &Stats, turn: u32) -> bool {
    turn >= stats.start_turn && (turn - stats.start_turn).is_multiple_of(stats.interval)
}

/// 戰場上的一隻精靈
#[derive(Clone, Debug)]
pub struct Unit {
    /// 種類代號，見 `Sprite::species`
    #[allow(dead_code)] // 精靈的圖與動畫尚未實作
    pub species: String,
    pub name: String,
    pub side: Side,
    pub pos: Pos,
    pub stats: Stats,
    pub skill: Skill,
    pub hp: u32,
    pub attacks_done: u32,
}

impl Unit {
    pub fn new(sprite: &Sprite, side: Side, pos: Pos) -> Self {
        assert!(
            pos.row < ROWS && pos.depth < DEPTHS,
            "位置超出棋盤：{pos:?}"
        );
        Self {
            species: sprite.species.clone(),
            name: sprite.name.clone(),
            side,
            pos,
            stats: sprite.stats,
            skill: sprite.skill.clone(),
            hp: sprite.stats.hp,
            attacks_done: 0,
        }
    }

    pub fn alive(&self) -> bool {
        self.hp > 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Win,
    Lose,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// 普攻或傷害技能；範圍技能每個目標各一個事件。`skill` 為 true 表示這次是技能
    Attack {
        attacker: usize,
        target: usize,
        damage: u32,
        skill: bool,
    },
    /// 治療技能；`amount` 是實際回復量（不超過最大 HP）
    Heal {
        healer: usize,
        target: usize,
        amount: u32,
    },
    Defeated {
        unit: usize,
    },
}

/// 一隻精靈這回合要做的事：目標與倍率（%）
enum Action {
    /// 傷害；最後的 bool 表示是否為技能
    Hit(Vec<usize>, u32, bool),
    Heal(Vec<usize>, u32),
}

pub struct Battle {
    pub units: Vec<Unit>,
    pub turn: u32,
    pub outcome: Option<Outcome>,
    rng: Rng,
}

impl Battle {
    pub fn new(units: Vec<Unit>, seed: u64) -> Self {
        let mut battle = Self {
            units,
            turn: 0,
            outcome: None,
            rng: Rng::new(seed),
        };
        battle.outcome = battle.check_outcome();
        battle
    }

    /// 進行一回合：行動者同時選目標，先結算治療，再結算傷害，最後才退場
    pub fn step(&mut self) -> Vec<Event> {
        if self.outcome.is_some() {
            return Vec::new();
        }
        self.turn += 1;
        let turn = self.turn;

        let actors: Vec<usize> = (0..self.units.len())
            .filter(|&i| self.units[i].alive() && acts_on(&self.units[i].stats, turn))
            .collect();

        // 1. 回合開始時，依當下戰場決定所有行動
        let mut actions: Vec<(usize, Action)> = Vec::new();
        for attacker in actors {
            let unit = &mut self.units[attacker];
            unit.attacks_done += 1;
            let skill = unit
                .attacks_done
                .is_multiple_of(unit.stats.skill_every)
                .then(|| unit.skill.clone());
            let action = match skill {
                Some(skill) => {
                    let targets =
                        targeting::skill_targets(&self.units, attacker, skill.range, &mut self.rng);
                    if skill.range == Range::Heal {
                        Action::Heal(targets, skill.power)
                    } else {
                        Action::Hit(targets, skill.power, true)
                    }
                }
                None => {
                    let target = targeting::pick_target(&self.units, attacker, &mut self.rng);
                    Action::Hit(target.into_iter().collect(), 100, false)
                }
            };
            actions.push((attacker, action));
        }

        // 2. 先結算所有治療
        let mut events = Vec::new();
        for (healer, action) in &actions {
            if let Action::Heal(targets, power) = action {
                let amount = heal_amount(self.units[*healer].stats.attack, *power);
                for &target in targets {
                    let t = &mut self.units[target];
                    let healed = amount.min(t.stats.hp - t.hp);
                    t.hp += healed;
                    events.push(Event::Heal {
                        healer: *healer,
                        target,
                        amount: healed,
                    });
                }
            }
        }

        // 3. 再結算所有傷害（目標在步驟 1 就決定好，所以可能打過頭）
        for (attacker, action) in &actions {
            if let Action::Hit(targets, power, skill) = action {
                let attack = self.units[*attacker].stats.attack;
                for &target in targets {
                    let dmg = skill_damage(attack, *power, self.units[target].stats.defense);
                    let t = &mut self.units[target];
                    t.hp = t.hp.saturating_sub(dmg);
                    events.push(Event::Attack {
                        attacker: *attacker,
                        target,
                        damage: dmg,
                        skill: *skill,
                    });
                }
            }
        }

        // 4. 結算完後，血量歸零的才退場
        let defeated: Vec<usize> = events
            .iter()
            .filter_map(|e| match *e {
                Event::Attack { target, .. } if !self.units[target].alive() => Some(target),
                _ => None,
            })
            .collect();
        let mut seen = Vec::new();
        for unit in defeated {
            if !seen.contains(&unit) {
                seen.push(unit);
                events.push(Event::Defeated { unit });
            }
        }

        self.outcome = self.check_outcome();
        events
    }

    /// 雙方同時全滅算輸
    fn check_outcome(&self) -> Option<Outcome> {
        let any_alive = |side| self.units.iter().any(|u| u.side == side && u.alive());
        if !any_alive(Side::Ally) {
            Some(Outcome::Lose)
        } else if !any_alive(Side::Enemy) {
            Some(Outcome::Win)
        } else {
            None
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn stats(hp: u32, attack: u32, defense: u32) -> Stats {
        Stats {
            hp,
            attack,
            defense,
            interval: 1,
            start_turn: 1,
            skill_every: 3,
        }
    }

    pub fn unit(side: Side, row: u8, depth: u8, s: Stats) -> Unit {
        Unit::new(&Sprite::new("u", s), side, Pos { row, depth })
    }

    /// 每次攻擊都放技能的精靈
    pub fn skilled(side: Side, row: u8, depth: u8, s: Stats, range: Range, power: u32) -> Unit {
        let skill = Skill {
            name: "t".to_owned(),
            range,
            power,
        };
        let sprite = Sprite::new(
            "u",
            Stats {
                skill_every: 1,
                ..s
            },
        )
        .with_skill(skill);
        Unit::new(&sprite, side, Pos { row, depth })
    }

    #[test]
    fn unit_keeps_species_id() {
        let sprite = Sprite::new("甲", stats(10, 1, 1)).with_species("ember_fox");
        let unit = Unit::new(&sprite, Side::Ally, Pos { row: 0, depth: 0 });
        assert_eq!(unit.species, "ember_fox");
    }

    #[test]
    fn rhythm_matches_design_example() {
        // 攻擊間隔 2、起始回合 3、技能次數 3
        let s = Stats {
            hp: 1000,
            attack: 1,
            defense: 0,
            interval: 2,
            start_turn: 3,
            skill_every: 3,
        };
        let turns: Vec<u32> = (1..=13).filter(|&t| acts_on(&s, t)).collect();
        assert_eq!(turns, vec![3, 5, 7, 9, 11, 13]);

        let me = unit(Side::Ally, 0, 0, s);
        let foe = unit(Side::Enemy, 0, 0, stats(1000, 1, 0));
        let mut b = Battle::new(vec![me, foe], 1);
        let mut skill_turns = Vec::new();
        for _ in 1..=13 {
            for e in b.step() {
                if let Event::Attack {
                    attacker: 0,
                    skill: true,
                    ..
                } = e
                {
                    skill_turns.push(b.turn);
                }
            }
        }
        assert_eq!(skill_turns, vec![7, 13]);
    }

    #[test]
    fn simultaneous_knockout_is_a_loss() {
        let a = unit(Side::Ally, 0, 0, stats(5, 100, 0));
        let e = unit(Side::Enemy, 0, 0, stats(5, 100, 0));
        let mut b = Battle::new(vec![a, e], 1);
        let events = b.step();
        assert!(events.contains(&Event::Defeated { unit: 0 }));
        assert!(events.contains(&Event::Defeated { unit: 1 }));
        assert_eq!(b.outcome, Some(Outcome::Lose));
    }

    #[test]
    fn targets_are_chosen_before_damage() {
        // 兩隻我方同時打同一個 1 HP 敵人，兩次攻擊都會發生（打過頭）
        let a1 = unit(Side::Ally, 0, 0, stats(10, 10, 0));
        let a2 = unit(Side::Ally, 0, 1, stats(10, 10, 0));
        let e = unit(Side::Enemy, 0, 0, stats(1, 0, 0));
        let mut b = Battle::new(vec![a1, a2, e], 1);
        let events = b.step();
        let hits = events
            .iter()
            .filter(|e| matches!(e, Event::Attack { target: 2, .. }))
            .count();
        assert_eq!(hits, 2);
        assert_eq!(b.outcome, Some(Outcome::Win));
    }

    #[test]
    fn dead_units_do_not_act() {
        let a = unit(Side::Ally, 0, 0, stats(10, 100, 0));
        let slow = Stats {
            start_turn: 2,
            ..stats(5, 100, 0)
        };
        let e1 = unit(Side::Enemy, 0, 0, slow);
        let e2 = unit(Side::Enemy, 2, 0, stats(1000, 1, 0));
        let mut b = Battle::new(vec![a, e1, e2], 1);
        b.step(); // e1 倒下
        let events = b.step();
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::Attack { attacker: 1, .. }))
        );
    }

    #[test]
    fn battle_ends_even_with_weak_attacks() {
        // 傷害至少 1，所以戰鬥一定會結束
        let a = unit(Side::Ally, 0, 0, stats(50, 1, 999));
        let e = unit(
            Side::Enemy,
            1,
            3,
            Stats {
                interval: 3,
                ..stats(40, 1, 999)
            },
        );
        let mut b = Battle::new(vec![a, e], 1);
        while b.outcome.is_none() {
            b.step();
            assert!(b.turn < 1000);
        }
        assert_eq!(b.outcome, Some(Outcome::Win));
    }

    #[test]
    fn heal_resolves_before_damage() {
        // 隊友只剩 10 HP，同一回合被打 15、被治療 20 → 先治療所以活下來
        let mut ally = unit(Side::Ally, 0, 0, stats(100, 0, 0));
        ally.hp = 10;
        ally.stats.start_turn = 99;
        let healer = skilled(Side::Ally, 2, 3, stats(100, 20, 0), Range::Heal, 100);
        let mut foe = unit(Side::Enemy, 0, 0, stats(1000, 15, 0));
        foe.stats.skill_every = 99;
        let mut b = Battle::new(vec![ally, healer, foe], 1);
        let events = b.step();
        assert!(events.contains(&Event::Heal {
            healer: 1,
            target: 0,
            amount: 20
        }));
        assert_eq!(b.units[0].hp, 15);
    }

    #[test]
    fn heal_is_capped_at_max_hp() {
        let mut healer = skilled(Side::Ally, 0, 0, stats(100, 50, 0), Range::Heal, 100);
        healer.hp = 90;
        let mut foe = unit(Side::Enemy, 2, 3, stats(1000, 1, 0));
        foe.stats.start_turn = 99;
        let mut b = Battle::new(vec![healer, foe], 1);
        let events = b.step();
        assert!(events.contains(&Event::Heal {
            healer: 0,
            target: 0,
            amount: 10
        }));
        assert_eq!(b.units[0].hp, 100);
    }

    #[test]
    fn area_skill_hits_each_target_with_power() {
        let a = skilled(Side::Ally, 0, 0, stats(100, 40, 0), Range::All, 50);
        let mut e1 = unit(Side::Enemy, 0, 0, stats(100, 1, 0));
        let mut e2 = unit(Side::Enemy, 2, 3, stats(100, 1, 100));
        e1.stats.start_turn = 99;
        e2.stats.start_turn = 99;
        let mut b = Battle::new(vec![a, e1, e2], 1);
        let events = b.step();
        assert_eq!(
            events,
            vec![
                Event::Attack {
                    attacker: 0,
                    target: 1,
                    damage: 20,
                    skill: true
                },
                Event::Attack {
                    attacker: 0,
                    target: 2,
                    damage: 10,
                    skill: true
                },
            ]
        );
    }

    #[test]
    fn units_carry_skill_from_data_file() {
        let s = crate::sprite::species::starter();
        let u = Unit::new(&s, Side::Ally, Pos { row: 0, depth: 0 });
        assert_eq!(u.skill, s.skill);
    }
}
