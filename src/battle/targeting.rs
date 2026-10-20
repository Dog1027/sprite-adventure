use super::{ROWS, Side, Unit};
use crate::rng::Rng;
use crate::sprite::skill::Range;

/// 普攻目標：同列最前面 → 最近的列最前面 → 剩餘血量比例低 → 隨機
pub fn pick_target(units: &[Unit], attacker: usize, rng: &mut Rng) -> Option<usize> {
    pick(&candidates(units, attacker), rng)
}

/// 擺位預覽：每隻我方精靈的第一個攻擊目標候選（平手時有多個）
pub fn preview(units: &[Unit]) -> Vec<(usize, Vec<usize>)> {
    (0..units.len())
        .filter(|&i| units[i].side == Side::Ally && units[i].alive())
        .map(|i| (i, candidates(units, i)))
        .collect()
}

/// 隨機之前的所有候選目標
fn candidates(units: &[Unit], attacker: usize) -> Vec<usize> {
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

    let Some(nearest) = fronts.iter().map(|&i| units[i].pos.row.abs_diff(row)).min() else {
        return Vec::new();
    };
    let mut candidates: Vec<usize> = fronts
        .into_iter()
        .filter(|&i| units[i].pos.row.abs_diff(row) == nearest)
        .collect();

    // 之後的「屬性有利優先」插在這裡

    lowest_hp_ratio(units, &mut candidates);
    candidates
}

/// 只留下剩餘血量比例最低的候選
/// （hp_a / max_a < hp_b / max_b ⇔ hp_a × max_b < hp_b × max_a）
fn lowest_hp_ratio(units: &[Unit], candidates: &mut Vec<usize>) {
    let key = |i: usize| (units[i].hp as u64, units[i].stats.hp as u64);
    let Some(lowest) = candidates
        .iter()
        .map(|&i| key(i))
        .min_by(|a, b| (a.0 * b.1).cmp(&(b.0 * a.1)))
    else {
        return;
    };
    candidates.retain(|&i| {
        let k = key(i);
        k.0 * lowest.1 == lowest.0 * k.1
    });
}

fn pick(candidates: &[usize], rng: &mut Rng) -> Option<usize> {
    (!candidates.is_empty()).then(|| candidates[rng.below(candidates.len())])
}

fn enemies_of(units: &[Unit], attacker: usize) -> Vec<usize> {
    let side = units[attacker].side;
    (0..units.len())
        .filter(|&i| units[i].alive() && units[i].side != side)
        .collect()
}

/// 技能的目標，規則見 DESIGN.md「技能」範圍表
pub fn skill_targets(units: &[Unit], attacker: usize, range: Range, rng: &mut Rng) -> Vec<usize> {
    let enemies = enemies_of(units, attacker);
    match range {
        Range::Single => pick_target(units, attacker, rng).into_iter().collect(),
        Range::Row => match pick_target(units, attacker, rng) {
            Some(t) => enemies
                .into_iter()
                .filter(|&i| units[i].pos.row == units[t].pos.row)
                .collect(),
            None => Vec::new(),
        },
        Range::Front => match enemies.iter().map(|&i| units[i].pos.depth).min() {
            Some(front) => enemies
                .into_iter()
                .filter(|&i| units[i].pos.depth == front)
                .collect(),
            None => Vec::new(),
        },
        Range::Snipe => {
            let back = enemies.iter().map(|&i| units[i].pos.depth).max();
            let mut candidates: Vec<usize> = enemies
                .into_iter()
                .filter(|&i| Some(units[i].pos.depth) == back)
                .collect();
            lowest_hp_ratio(units, &mut candidates);
            pick(&candidates, rng).into_iter().collect()
        }
        Range::All => enemies,
        Range::Heal => {
            let side = units[attacker].side;
            let mut candidates: Vec<usize> = (0..units.len())
                .filter(|&i| units[i].alive() && units[i].side == side)
                .collect();
            lowest_hp_ratio(units, &mut candidates);
            pick(&candidates, rng).into_iter().collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn preview_lists_tied_candidates() {
        let a = unit(Side::Ally, 1, 0, stats(10, 1, 0));
        let b = unit(Side::Ally, 0, 2, stats(10, 1, 0));
        let x = unit(Side::Enemy, 0, 0, stats(10, 1, 0));
        let y = unit(Side::Enemy, 2, 0, stats(10, 1, 0));
        let units = [a, b, x, y];
        assert_eq!(preview(&units), vec![(0, vec![2, 3]), (1, vec![2])]);
    }

    // 敵方棋盤（depth 0 為最前方）：
    //   第1列: e1(depth 1)          e4(depth 3, HP 50%)
    //   第2列: e2(depth 0)
    //   第3列: e3(depth 1)          e5(depth 3)
    fn skill_board() -> Vec<Unit> {
        let mut e4 = unit(Side::Enemy, 0, 3, stats(10, 1, 0));
        e4.hp = 5;
        vec![
            unit(Side::Ally, 0, 0, stats(10, 1, 0)), // 0 施放者，在第 1 列
            unit(Side::Enemy, 0, 1, stats(10, 1, 0)), // 1
            unit(Side::Enemy, 1, 0, stats(10, 1, 0)), // 2
            unit(Side::Enemy, 2, 1, stats(10, 1, 0)), // 3
            e4,                                      // 4
            unit(Side::Enemy, 2, 3, stats(10, 1, 0)), // 5
        ]
    }

    fn targets(units: &[Unit], range: Range) -> Vec<usize> {
        skill_targets(units, 0, range, &mut Rng::new(1))
    }

    #[test]
    fn skill_single_is_normal_target() {
        assert_eq!(targets(&skill_board(), Range::Single), vec![1]);
    }

    #[test]
    fn skill_row_hits_whole_row_of_normal_target() {
        assert_eq!(targets(&skill_board(), Range::Row), vec![1, 4]);
    }

    #[test]
    fn skill_front_hits_frontmost_depth() {
        assert_eq!(targets(&skill_board(), Range::Front), vec![2]);
    }

    #[test]
    fn skill_snipe_hits_backmost_lowest_hp_ratio() {
        assert_eq!(targets(&skill_board(), Range::Snipe), vec![4]);
    }

    #[test]
    fn skill_all_hits_every_enemy() {
        assert_eq!(targets(&skill_board(), Range::All), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn skill_heal_picks_lowest_ally_including_self() {
        let mut units = skill_board();
        let mut friend = unit(Side::Ally, 1, 0, stats(10, 1, 0));
        friend.hp = 6;
        units.push(friend); // 6
        assert_eq!(targets(&units, Range::Heal), vec![6]);
        units[0].hp = 3;
        assert_eq!(targets(&units, Range::Heal), vec![0]);
    }

    #[test]
    fn skills_ignore_dead_units() {
        let mut units = skill_board();
        units[2].hp = 0;
        units[1].hp = 0;
        assert_eq!(targets(&units, Range::Front), vec![3]);
        assert_eq!(targets(&units, Range::All), vec![3, 4, 5]);
    }
}
