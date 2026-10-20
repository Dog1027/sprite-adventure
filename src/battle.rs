//! 戰鬥核心邏輯（不含畫面），規則見 docs/DESIGN.md「戰鬥」。

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

impl Stats {
    pub fn acts_on(&self, turn: u32) -> bool {
        turn >= self.start_turn && (turn - self.start_turn).is_multiple_of(self.interval)
    }
}

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
    pub fn new(name: &str, side: Side, pos: Pos, stats: Stats) -> Self {
        assert!(
            pos.row < ROWS && pos.depth < DEPTHS,
            "位置超出棋盤：{pos:?}"
        );
        assert!(stats.interval >= 1 && stats.start_turn >= 1 && stats.skill_every >= 1);
        Self {
            name: name.to_owned(),
            side,
            pos,
            stats,
            hp: stats.hp,
            attacks_done: 0,
        }
    }

    pub fn alive(&self) -> bool {
        self.hp > 0
    }
}

/// 傷害計算集中在這裡：攻擊 × 100 ÷ (100 + 防禦)，四捨五入，至少 1。
/// 之後的屬性倍率、陣型加成也在這裡乘上去。
pub fn damage(attack: u32, defense: u32) -> u32 {
    let den = 100 + defense as u64;
    let rounded = (attack as u64 * 200 + den) / (den * 2);
    rounded.max(1) as u32
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

/// 簡單的 xorshift 亂數，讓戰鬥可以用種子重現
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    pub fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
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

    /// 普攻目標：同列最前面 → 最近的列最前面 → 剩餘血量比例低 → 隨機
    pub fn pick_target(&mut self, attacker: usize) -> Option<usize> {
        let me = &self.units[attacker];
        let row = me.pos.row;
        let side = me.side;

        // 每一列最前面的敵人
        let mut fronts: Vec<usize> = Vec::new();
        for r in 0..ROWS {
            let front = (0..self.units.len())
                .filter(|&i| {
                    let u = &self.units[i];
                    u.alive() && u.side != side && u.pos.row == r
                })
                .min_by_key(|&i| self.units[i].pos.depth);
            fronts.extend(front);
        }

        let nearest = fronts
            .iter()
            .map(|&i| self.units[i].pos.row.abs_diff(row))
            .min()?;
        let mut candidates: Vec<usize> = fronts
            .into_iter()
            .filter(|&i| self.units[i].pos.row.abs_diff(row) == nearest)
            .collect();

        // 之後的「屬性有利優先」插在這裡

        // 剩餘血量比例低者優先：hp_a / max_a < hp_b / max_b ⇔ hp_a × max_b < hp_b × max_a
        let ratio_key = |u: &Unit| (u.hp as u64, u.stats.hp as u64);
        let lowest = candidates
            .iter()
            .map(|&i| ratio_key(&self.units[i]))
            .min_by(|a, b| (a.0 * b.1).cmp(&(b.0 * a.1)))?;
        candidates.retain(|&i| {
            let k = ratio_key(&self.units[i]);
            k.0 * lowest.1 == lowest.0 * k.1
        });

        Some(candidates[self.rng.below(candidates.len())])
    }

    /// 進行一回合：行動者同時選目標，先結算治療，再結算傷害，最後才退場
    pub fn step(&mut self) -> Vec<Event> {
        if self.outcome.is_some() {
            return Vec::new();
        }
        self.turn += 1;
        let turn = self.turn;

        let actors: Vec<usize> = (0..self.units.len())
            .filter(|&i| self.units[i].alive() && self.units[i].stats.acts_on(turn))
            .collect();

        let mut events = Vec::new();
        for attacker in actors {
            let Some(target) = self.pick_target(attacker) else {
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
mod tests {
    use super::*;

    fn stats(hp: u32, attack: u32, defense: u32) -> Stats {
        Stats {
            hp,
            attack,
            defense,
            interval: 1,
            start_turn: 1,
            skill_every: 3,
        }
    }

    fn unit(side: Side, row: u8, depth: u8, s: Stats) -> Unit {
        Unit::new("u", side, Pos { row, depth }, s)
    }

    #[test]
    fn damage_formula() {
        assert_eq!(damage(100, 0), 100);
        assert_eq!(damage(100, 100), 50);
        assert_eq!(damage(10, 50), 7); // 6.67
        assert_eq!(damage(3, 100), 2); // 1.5 四捨五入
        assert_eq!(damage(1, 1000), 1); // 至少 1
        assert_eq!(damage(0, 0), 1);
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
        let turns: Vec<u32> = (1..=13).filter(|&t| s.acts_on(t)).collect();
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
    fn targets_front_of_same_row() {
        let me = unit(Side::Ally, 1, 0, stats(10, 1, 0));
        let back = unit(Side::Enemy, 1, 2, stats(10, 1, 0));
        let front = unit(Side::Enemy, 1, 1, stats(10, 1, 0));
        let other_row = unit(Side::Enemy, 0, 0, stats(10, 1, 0));
        let mut b = Battle::new(vec![me, back, front, other_row], 1);
        assert_eq!(b.pick_target(0), Some(2));
    }

    #[test]
    fn targets_nearest_row() {
        let me = unit(Side::Ally, 0, 0, stats(10, 1, 0));
        let near = unit(Side::Enemy, 1, 3, stats(10, 1, 0));
        let far = unit(Side::Enemy, 2, 0, stats(10, 1, 0));
        let mut b = Battle::new(vec![me, near, far], 1);
        assert_eq!(b.pick_target(0), Some(1));
    }

    #[test]
    fn equal_rows_prefer_lower_hp_ratio() {
        // 設計文件範例：A 在第 2 列，X 在第 1 列、Y 在第 3 列
        let a = unit(Side::Ally, 1, 0, stats(10, 1, 0));
        let mut x = unit(Side::Enemy, 0, 0, stats(100, 1, 0));
        let mut y = unit(Side::Enemy, 2, 2, stats(10, 1, 0));
        x.hp = 40; // 40%
        y.hp = 3; // 30%
        let mut b = Battle::new(vec![a, x, y], 1);
        assert_eq!(b.pick_target(0), Some(2));
    }

    #[test]
    fn full_tie_is_random_between_candidates() {
        let mut seen = [false; 3];
        for seed in 1..50 {
            let a = unit(Side::Ally, 1, 0, stats(10, 1, 0));
            let x = unit(Side::Enemy, 0, 0, stats(10, 1, 0));
            let y = unit(Side::Enemy, 2, 0, stats(10, 1, 0));
            let mut b = Battle::new(vec![a, x, y], seed);
            seen[b.pick_target(0).unwrap()] = true;
        }
        assert_eq!(seen, [false, true, true]);
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
