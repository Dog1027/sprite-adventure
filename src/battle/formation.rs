//! 陣型：開放哪些格子、陣型加成、把精靈擺上棋盤。規則見 docs/DESIGN.md「陣型」。

use super::{DEPTHS, Pos, ROWS, Side, Unit};
use crate::sprite::Sprite;

/// 一場戰鬥最少、最多上場隻數
pub const MIN_UNITS: usize = 3;
pub const MAX_SLOTS: usize = 6;

#[allow(dead_code)] // 各陣型的具體效果尚未決定
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BonusTarget {
    /// 全隊加成
    All,
    /// 格子專屬加成（陣型的第幾格）
    Slot(usize),
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BonusStat {
    Hp,
    Attack,
    Defense,
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
            (MIN_UNITS..=MAX_SLOTS).contains(&slots.len()),
            "陣型 {name} 有 {} 格，必須是 {MIN_UNITS}～{MAX_SLOTS} 格",
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
                for b in &self.bonuses {
                    if b.target == BonusTarget::All || b.target == BonusTarget::Slot(i) {
                        apply(&mut unit, b);
                    }
                }
                unit
            })
            .collect())
    }
}

fn apply(unit: &mut Unit, bonus: &Bonus) {
    let scale = |v: u32| ((v as u64 * (100 + bonus.percent as u64) + 50) / 100) as u32;
    let s = &mut unit.stats;
    match bonus.stat {
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

/// 範例陣型（形狀取自 DESIGN.md，名稱僅為示意，效果尚未決定）。之後改讀資料檔。
pub fn samples() -> Vec<Formation> {
    vec![
        Formation::from_rows("Arrow", ["..X.", ".XXX", "..X."], vec![]),
        Formation::from_rows("Square", ["..XX", ".X.X", "..XX"], vec![]),
        Formation::from_rows("Geese", ["X...", ".X..", "..X."], vec![]),
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
    fn needs_at_least_three_units() {
        let f = &samples()[1];
        let s = sprite();
        let two = [Some(&s), None, Some(&s)];
        assert_eq!(
            f.place(&two, Side::Ally).unwrap_err(),
            PlaceError::TooFew(2)
        );
        // 不用擺滿
        let three = [Some(&s), None, Some(&s), Some(&s)];
        assert_eq!(f.place(&three, Side::Ally).unwrap().len(), 3);
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
}
