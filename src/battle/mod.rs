//! 戰鬥：棋盤、回合節奏、目標選擇、傷害與結算。規則見 docs/DESIGN.md「陣型」「戰鬥」。
//!
//! 對外介面：用 `Unit::new` 把精靈擺上棋盤，`Battle::new` 開始戰鬥，
//! 反覆呼叫 `Battle::step` 直到 `outcome` 有結果。畫面在 `view`。

mod damage;
pub mod formation;
mod targeting;

pub use targeting::preview;
pub mod view;

pub use damage::damage;

use crate::rng::Rng;
use crate::sprite::{Sprite, Stats};

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
    pub name: String,
    pub side: Side,
    pub pos: Pos,
    pub stats: Stats,
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
            name: sprite.name.clone(),
            side,
            pos,
            stats: sprite.stats,
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
    /// `skill` 為 true 表示這次輪到技能（技能尚未實作，暫時以普攻代替）
    Attack {
        attacker: usize,
        target: usize,
        damage: u32,
        skill: bool,
    },
    Defeated {
        unit: usize,
    },
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

        let mut events = Vec::new();
        for attacker in actors {
            let Some(target) = targeting::pick_target(&self.units, attacker, &mut self.rng) else {
                continue;
            };
            let defense = self.units[target].stats.defense;
            let unit = &mut self.units[attacker];
            unit.attacks_done += 1;
            let skill = unit.attacks_done.is_multiple_of(unit.stats.skill_every);
            let dmg = damage(unit.stats.attack, defense);
            events.push(Event::Attack {
                attacker,
                target,
                damage: dmg,
                skill,
            });
        }

        // 治療：技能尚未實作，之後在此先結算

        for e in &events {
            if let Event::Attack { target, damage, .. } = *e {
                let t = &mut self.units[target];
                t.hp = t.hp.saturating_sub(damage);
            }
        }

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
}
