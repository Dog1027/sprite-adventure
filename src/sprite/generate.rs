//! 生成精靈：捕捉節點出現哪些精靈、敵方出現哪些精靈。規則見 docs/DESIGN.md「精靈」。
//!
//! 依稀有度加權隨機挑選，越後面的階段越容易出現稀有精靈；敵方的能力值隨階段提升。
//! 第 1 個階段只出現普通精靈，敵方也不會是開局精靈（雙方一樣時一定同歸於盡，算我方輸）。
//! 同一個亂數種子會得到同樣的結果。

use super::Sprite;
use super::species::{self, Rarity};
use crate::rng::Rng;

/// 敵方每過一個階段，HP、攻擊、防禦提升的百分比；暫定
pub const ENEMY_SCALE_PERCENT: u32 = 5;

/// 稀有度在某個階段（從 0 起算）的出現權重；暫定
fn weight(rarity: Rarity, stage: usize) -> usize {
    match rarity {
        Rarity::Common => 6,
        // 少見與稀有在第 1 個階段不出現，之後逐漸增加
        Rarity::Uncommon if stage == 0 => 0,
        Rarity::Uncommon => 2 + stage / 2,
        Rarity::Rare => stage,
    }
}

/// 從候選中依權重挑一個，回傳在 `candidates` 中的位置；權重全為 0 時回傳 None
fn pick(candidates: &[(&'static str, usize)], rng: &mut Rng) -> Option<usize> {
    let total: usize = candidates.iter().map(|&(_, w)| w).sum();
    if total == 0 {
        return None;
    }
    let mut roll = rng.below(total);
    candidates.iter().position(|&(_, w)| {
        if roll < w {
            true
        } else {
            roll -= w;
            false
        }
    })
}

fn candidates(stage: usize, include_starter: bool) -> Vec<(&'static str, usize)> {
    species::all()
        .filter(|(_, s)| include_starter || !s.starter)
        .map(|(id, s)| (id, weight(s.rarity, stage)))
        .collect()
}

/// 捕捉節點出現的精靈：`count` 種不重複的代號（用 `species::create`、`species::catch_rate` 查詢）
pub fn catch_offers(rng: &mut Rng, stage: usize, count: usize) -> Vec<&'static str> {
    let mut pool = candidates(stage, true);
    let mut offers = Vec::new();
    while offers.len() < count {
        let Some(i) = pick(&pool, rng) else { break };
        offers.push(pool.remove(i).0);
    }
    offers
}

/// 一隻敵方精靈：依權重挑種類（不含開局精靈），能力值隨階段提升（節奏數值不變）
pub fn enemy(rng: &mut Rng, stage: usize) -> Sprite {
    let pool = candidates(stage, false);
    let i = pick(&pool, rng).expect("一定有普通精靈");
    let mut sprite = species::create(pool[i].0);
    let percent = 100 + ENEMY_SCALE_PERCENT * stage as u32;
    for v in [
        &mut sprite.stats.hp,
        &mut sprite.stats.attack,
        &mut sprite.stats.defense,
    ] {
        *v = (*v * percent + 50) / 100;
    }
    sprite
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rarity_of(id: &str) -> Rarity {
        species::all().find(|(i, _)| *i == id).unwrap().1.rarity
    }

    #[test]
    fn offers_are_distinct() {
        let mut rng = Rng::new(1);
        for stage in 0..8 {
            let offers = catch_offers(&mut rng, stage, 3);
            assert_eq!(offers.len(), 3);
            for o in &offers {
                assert_eq!(offers.iter().filter(|x| *x == o).count(), 1);
            }
        }
    }

    #[test]
    fn offers_stop_when_pool_runs_out() {
        let mut rng = Rng::new(1);
        // 第 1 個階段只有普通精靈的權重不是 0
        let n = species::all()
            .filter(|(_, s)| s.rarity == Rarity::Common)
            .count();
        assert_eq!(catch_offers(&mut rng, 0, 100).len(), n);
    }

    #[test]
    fn only_common_at_first_stage() {
        let mut rng = Rng::new(7);
        let mut rare_later = 0;
        for _ in 0..300 {
            for id in catch_offers(&mut rng, 0, 3) {
                assert_eq!(rarity_of(id), Rarity::Common);
            }
            let late = catch_offers(&mut rng, 7, 3);
            rare_later += late
                .iter()
                .filter(|id| rarity_of(id) == Rarity::Rare)
                .count();
        }
        assert!(rare_later > 0);
    }

    #[test]
    fn enemy_is_never_the_starter() {
        let starter = species::starter().name;
        let mut rng = Rng::new(9);
        for stage in 0..8 {
            for _ in 0..100 {
                assert_ne!(enemy(&mut rng, stage).name, starter);
            }
        }
    }

    #[test]
    fn same_seed_same_result() {
        let a = catch_offers(&mut Rng::new(42), 3, 3);
        let b = catch_offers(&mut Rng::new(42), 3, 3);
        assert_eq!(a, b);
    }

    #[test]
    fn enemy_scales_with_stage() {
        // 用同一個種子，兩個階段挑到的種類可能不同，所以比較同一種的基本值
        let mut rng = Rng::new(3);
        let e0 = enemy(&mut rng, 0);
        let base = species::all()
            .map(|(_, s)| &s.sprite)
            .find(|s| s.name == e0.name)
            .unwrap();
        assert_eq!(e0.stats, base.stats);

        let mut rng = Rng::new(3);
        for _ in 0..50 {
            let e = enemy(&mut rng, 4);
            let base = species::all()
                .map(|(_, s)| &s.sprite)
                .find(|s| s.name == e.name)
                .unwrap();
            assert_eq!(e.stats.hp, (base.stats.hp * 120 + 50) / 100);
            assert_eq!(e.stats.interval, base.stats.interval);
        }
    }
}
