use super::{ROWS, Unit};
use crate::rng::Rng;

/// 普攻目標：同列最前面 → 最近的列最前面 → 剩餘血量比例低 → 隨機
pub fn pick_target(units: &[Unit], attacker: usize, rng: &mut Rng) -> Option<usize> {
    let me = &units[attacker];
    let row = me.pos.row;
    let side = me.side;

    // 每一列最前面的敵人
    let mut fronts: Vec<usize> = Vec::new();
    for r in 0..ROWS {
        let front = (0..units.len())
            .filter(|&i| {
                let u = &units[i];
                u.alive() && u.side != side && u.pos.row == r
            })
            .min_by_key(|&i| units[i].pos.depth);
        fronts.extend(front);
    }

    let nearest = fronts
        .iter()
        .map(|&i| units[i].pos.row.abs_diff(row))
        .min()?;
    let mut candidates: Vec<usize> = fronts
        .into_iter()
        .filter(|&i| units[i].pos.row.abs_diff(row) == nearest)
        .collect();

    // 之後的「屬性有利優先」插在這裡

    // 剩餘血量比例低者優先：hp_a / max_a < hp_b / max_b ⇔ hp_a × max_b < hp_b × max_a
    let ratio_key = |u: &Unit| (u.hp as u64, u.stats.hp as u64);
    let lowest = candidates
        .iter()
        .map(|&i| ratio_key(&units[i]))
        .min_by(|a, b| (a.0 * b.1).cmp(&(b.0 * a.1)))?;
    candidates.retain(|&i| {
        let k = ratio_key(&units[i]);
        k.0 * lowest.1 == lowest.0 * k.1
    });

    Some(candidates[rng.below(candidates.len())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::Side;
    use crate::battle::tests::{stats, unit};

    #[test]
    fn targets_front_of_same_row() {
        let me = unit(Side::Ally, 1, 0, stats(10, 1, 0));
        let back = unit(Side::Enemy, 1, 2, stats(10, 1, 0));
        let front = unit(Side::Enemy, 1, 1, stats(10, 1, 0));
        let other_row = unit(Side::Enemy, 0, 0, stats(10, 1, 0));
        let units = [me, back, front, other_row];
        assert_eq!(pick_target(&units, 0, &mut Rng::new(1)), Some(2));
    }

    #[test]
    fn targets_nearest_row() {
        let me = unit(Side::Ally, 0, 0, stats(10, 1, 0));
        let near = unit(Side::Enemy, 1, 3, stats(10, 1, 0));
        let far = unit(Side::Enemy, 2, 0, stats(10, 1, 0));
        let units = [me, near, far];
        assert_eq!(pick_target(&units, 0, &mut Rng::new(1)), Some(1));
    }

    #[test]
    fn equal_rows_prefer_lower_hp_ratio() {
        // 設計文件範例：A 在第 2 列，X 在第 1 列、Y 在第 3 列
        let a = unit(Side::Ally, 1, 0, stats(10, 1, 0));
        let mut x = unit(Side::Enemy, 0, 0, stats(100, 1, 0));
        let mut y = unit(Side::Enemy, 2, 2, stats(10, 1, 0));
        x.hp = 40; // 40%
        y.hp = 3; // 30%
        let units = [a, x, y];
        assert_eq!(pick_target(&units, 0, &mut Rng::new(1)), Some(2));
    }

    #[test]
    fn full_tie_is_random_between_candidates() {
        let a = unit(Side::Ally, 1, 0, stats(10, 1, 0));
        let x = unit(Side::Enemy, 0, 0, stats(10, 1, 0));
        let y = unit(Side::Enemy, 2, 0, stats(10, 1, 0));
        let units = [a, x, y];
        let mut seen = [false; 3];
        for seed in 1..50 {
            seen[pick_target(&units, 0, &mut Rng::new(seed)).unwrap()] = true;
        }
        assert_eq!(seen, [false, true, true]);
    }

    #[test]
    fn no_enemies_left() {
        let units = [unit(Side::Ally, 0, 0, stats(10, 1, 0))];
        assert_eq!(pick_target(&units, 0, &mut Rng::new(1)), None);
    }
}
