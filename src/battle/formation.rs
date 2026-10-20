//! 陣型：開放哪些格子、陣型加成、把精靈擺上棋盤。規則見 docs/DESIGN.md「陣型」。

use super::{DEPTHS, Pos, ROWS, Side, Unit};
use crate::sprite::Sprite;

/// 一場戰鬥最少上場隻數
pub const MIN_UNITS: usize = 1;
/// 每個陣型開放的格子數
pub const MIN_SLOTS: usize = 3;
pub const MAX_SLOTS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BonusTarget {
    /// 全隊加成
    All,
    /// 格子專屬加成（陣型的第幾格）
    Slot(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BonusStat {
    Hp,
    Attack,
    Defense,
}

impl BonusStat {
    pub const ALL: [BonusStat; 3] = [BonusStat::Hp, BonusStat::Attack, BonusStat::Defense];

    /// 格子上顯示用的短名稱
    pub fn short_label(self) -> &'static str {
        match self {
            BonusStat::Hp => "HP",
            BonusStat::Attack => "攻",
            BonusStat::Defense => "防",
        }
    }
}

/// 格子加成的簡短說明，例如「HP+20 攻+20」；沒有加成時為空字串
pub fn bonus_label(percents: &[(BonusStat, u32)]) -> String {
    percents
        .iter()
        .map(|(stat, p)| format!("{}+{p}", stat.short_label()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// 陣型加成，例如全員防禦 +15%：`Bonus { target: All, stat: Defense, percent: 15 }`
/// 不能改變節奏數值（見 DESIGN.md）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bonus {
    pub target: BonusTarget,
    pub stat: BonusStat,
    pub percent: u32,
}

#[derive(Clone, Debug)]
pub struct Formation {
    pub name: String,
    /// 開放的格子
    pub slots: Vec<Pos>,
    pub bonuses: Vec<Bonus>,
}

impl Formation {
    /// 以我方視角的圖形建立陣型：每個字串是一列（第 1～3 列），
    /// 由左到右是後方到前方，`X` 為開放的格子
    pub fn from_rows(name: &str, rows: [&str; ROWS as usize], bonuses: Vec<Bonus>) -> Self {
        let mut slots = Vec::new();
        for (r, line) in rows.iter().enumerate() {
            assert_eq!(
                line.len(),
                DEPTHS as usize,
                "陣型 {name} 第 {} 列長度錯誤",
                r + 1
            );
            for (c, ch) in line.chars().enumerate() {
                if ch == 'X' {
                    slots.push(Pos {
                        row: r as u8,
                        depth: DEPTHS - 1 - c as u8,
                    });
                }
            }
        }
        assert!(
            (MIN_SLOTS..=MAX_SLOTS).contains(&slots.len()),
            "陣型 {name} 有 {} 格，必須是 {MIN_SLOTS}～{MAX_SLOTS} 格",
            slots.len()
        );
        for b in &bonuses {
            if let BonusTarget::Slot(i) = b.target {
                assert!(i < slots.len(), "陣型 {name} 的加成指向不存在的第 {i} 格");
            }
        }
        Self {
            name: name.to_owned(),
            slots,
            bonuses,
        }
    }

    /// 第 i 格的加成：全隊加成與格子加成**相加**（攻擊 +10% 與 +20% 合計 +30%），
    /// 依 HP、攻擊、防禦的順序，省略沒有加成的項目
    pub fn slot_percents(&self, i: usize) -> Vec<(BonusStat, u32)> {
        BonusStat::ALL
            .into_iter()
            .map(|stat| {
                let total = self
                    .bonuses
                    .iter()
                    .filter(|b| b.stat == stat)
                    .filter(|b| b.target == BonusTarget::All || b.target == BonusTarget::Slot(i))
                    .map(|b| b.percent)
                    .sum();
                (stat, total)
            })
            .filter(|&(_, total)| total > 0)
            .collect()
    }

    /// 把精靈擺到陣型的格子上。`placement[i]` 是放在第 i 格的精靈（`None` 為空格）。
    /// 陣型加成在這裡套用到 `Unit` 的能力值。
    pub fn place(
        &self,
        placement: &[Option<&Sprite>],
        side: Side,
    ) -> Result<Vec<Unit>, PlaceError> {
        if placement.len() > self.slots.len() {
            return Err(PlaceError::TooManySlots);
        }
        let count = placement.iter().flatten().count();
        if count < MIN_UNITS {
            return Err(PlaceError::TooFew(count));
        }
        Ok(placement
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.map(|s| (i, s)))
            .map(|(i, sprite)| {
                let mut unit = Unit::new(sprite, side, self.slots[i]);
                for (stat, percent) in self.slot_percents(i) {
                    apply(&mut unit, stat, percent);
                }
                unit
            })
            .collect())
    }
}

/// 把一項能力值提升 percent%，四捨五入
fn apply(unit: &mut Unit, stat: BonusStat, percent: u32) {
    let scale = |v: u32| ((v as u64 * (100 + percent as u64) + 50) / 100) as u32;
    let s = &mut unit.stats;
    match stat {
        BonusStat::Hp => {
            s.hp = scale(s.hp);
            unit.hp = s.hp;
        }
        BonusStat::Attack => s.attack = scale(s.attack),
        BonusStat::Defense => s.defense = scale(s.defense),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceError {
    /// 上場隻數不足
    TooFew(usize),
    /// 指定的格子比陣型開放的多
    TooManySlots,
}

fn all(stat: BonusStat, percent: u32) -> Bonus {
    Bonus {
        target: BonusTarget::All,
        stat,
        percent,
    }
}

fn slot(i: usize, stat: BonusStat, percent: u32) -> Bonus {
    Bonus {
        target: BonusTarget::Slot(i),
        stat,
        percent,
    }
}

/// 第一版的陣型（形狀與加成見 DESIGN.md「陣型」，數值暫定）。之後改讀資料檔。
/// 格子編號依 `from_rows` 的順序：由上到下、每列由後到前。
pub fn samples() -> Vec<Formation> {
    use BonusStat::*;
    vec![
        // 進攻：全隊攻擊，再依位置加成。格子：0 上翼、1 尾、2 心、3 尖、4 下翼
        Formation::from_rows(
            "鋒矢陣",
            ["..X.", ".XXX", "..X."],
            vec![
                all(Attack, 10),
                slot(3, Attack, 20),
                slot(0, Defense, 15),
                slot(4, Defense, 15),
                slot(1, Hp, 15),
            ],
        ),
        // 防守：只有全隊加成，上場最多
        Formation::from_rows("方圓陣", ["..XX", ".X.X", "..XX"], vec![all(Defense, 15)]),
        // 人少但每格都強。格子：0 後、1 中、2 前
        Formation::from_rows(
            "雁行陣",
            ["X...", ".X..", "..X."],
            vec![
                slot(0, Attack, 40),
                slot(1, Hp, 20),
                slot(1, Attack, 20),
                slot(2, Defense, 40),
            ],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::tests::stats;

    fn sprite() -> Sprite {
        Sprite::new("s", stats(100, 20, 10))
    }

    #[test]
    fn shape_maps_right_side_to_front() {
        let f = Formation::from_rows("t", ["X..X", "...X", "...X"], vec![]);
        assert_eq!(
            f.slots,
            vec![
                Pos { row: 0, depth: 3 },
                Pos { row: 0, depth: 0 },
                Pos { row: 1, depth: 0 },
                Pos { row: 2, depth: 0 },
            ]
        );
    }

    #[test]
    fn sample_formations_are_valid() {
        let sizes: Vec<usize> = samples().iter().map(|f| f.slots.len()).collect();
        assert_eq!(sizes, vec![5, 6, 3]);
    }

    #[test]
    #[should_panic]
    fn too_few_slots_rejected() {
        Formation::from_rows("t", ["X...", "X...", "...."], vec![]);
    }

    #[test]
    fn needs_at_least_one_unit() {
        let f = &samples()[1];
        let s = sprite();
        let none = [None, None, None];
        assert_eq!(
            f.place(&none, Side::Ally).unwrap_err(),
            PlaceError::TooFew(0)
        );
        // 不用擺滿
        let one = [None, Some(&s)];
        assert_eq!(f.place(&one, Side::Ally).unwrap().len(), 1);
    }

    #[test]
    fn bonuses_apply_to_all_or_one_slot() {
        let f = Formation::from_rows(
            "t",
            ["...X", "...X", "...X"],
            vec![
                Bonus {
                    target: BonusTarget::All,
                    stat: BonusStat::Defense,
                    percent: 15,
                },
                Bonus {
                    target: BonusTarget::Slot(0),
                    stat: BonusStat::Attack,
                    percent: 30,
                },
            ],
        );
        let s = sprite();
        let units = f
            .place(&[Some(&s), Some(&s), Some(&s)], Side::Ally)
            .unwrap();
        assert_eq!(units[0].stats.attack, 26);
        assert_eq!(units[1].stats.attack, 20);
        assert!(units.iter().all(|u| u.stats.defense == 12)); // 11.5 四捨五入
    }

    #[test]
    fn bonuses_add_up_instead_of_multiplying() {
        let f = Formation::from_rows(
            "t",
            ["...X", "...X", "...X"],
            vec![all(BonusStat::Attack, 10), slot(0, BonusStat::Attack, 20)],
        );
        let s = sprite();
        let units = f.place(&[Some(&s)], Side::Ally).unwrap();
        // 20 × 1.3 = 26（相乘的話是 20 × 1.1 × 1.2 = 26.4）
        assert_eq!(units[0].stats.attack, 26);
        assert_eq!(f.slot_percents(0), vec![(BonusStat::Attack, 30)]);
        assert_eq!(f.slot_percents(1), vec![(BonusStat::Attack, 10)]);
    }

    #[test]
    fn sample_bonuses_match_design() {
        let f = samples();
        let labels = |f: &Formation| -> Vec<String> {
            (0..f.slots.len())
                .map(|i| bonus_label(&f.slot_percents(i)))
                .collect()
        };
        assert_eq!(
            labels(&f[0]),
            vec![
                "攻+10 防+15",
                "HP+15 攻+10",
                "攻+10",
                "攻+30",
                "攻+10 防+15"
            ]
        );
        assert!(labels(&f[1]).iter().all(|l| l == "防+15"));
        assert_eq!(labels(&f[2]), vec!["攻+40", "HP+20 攻+20", "防+40"]);
    }

    #[test]
    fn hp_bonus_also_fills_current_hp() {
        let f = &samples()[2];
        let s = sprite();
        let units = f.place(&[None, Some(&s)], Side::Ally).unwrap();
        assert_eq!((units[0].stats.hp, units[0].hp), (120, 120));
    }
}
