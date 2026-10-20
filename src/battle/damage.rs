/// 傷害計算集中在這裡：攻擊 × 倍率 ÷ (100 + 防禦)，四捨五入，至少 1。
/// 普攻的倍率是 100（即 攻擊 × 100 ÷ (100 + 防禦)）；技能以「攻擊 × 倍率」代入同一個公式，
/// 一次算完再四捨五入，避免中間先四捨五入造成誤差。
/// 之後的屬性倍率也在這裡乘上去；陣型加成在擺位時已套用到能力值（見 formation.rs）。
pub fn skill_damage(attack: u32, power: u32, defense: u32) -> u32 {
    let num = attack as u64 * power as u64;
    let den = 100 + defense as u64;
    let rounded = (num * 2 + den) / (den * 2);
    rounded.max(1) as u32
}

/// 治療量：攻擊 × 倍率，四捨五入（上限由呼叫端以最大 HP 限制）
pub fn heal_amount(attack: u32, power: u32) -> u32 {
    ((attack as u64 * power as u64 + 50) / 100) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damage_formula() {
        let damage = |attack, defense| skill_damage(attack, 100, defense);
        assert_eq!(damage(100, 0), 100);
        assert_eq!(damage(100, 100), 50);
        assert_eq!(damage(10, 50), 7); // 6.67
        assert_eq!(damage(3, 100), 2); // 1.5 四捨五入
        assert_eq!(damage(1, 1000), 1); // 至少 1
        assert_eq!(damage(0, 0), 1);
    }

    #[test]
    fn skill_damage_uses_power() {
        assert_eq!(skill_damage(30, 200, 50), 40); // 60 × 100 ÷ 150
        assert_eq!(skill_damage(35, 50, 0), 18); // 17.5 四捨五入
        assert_eq!(skill_damage(1, 50, 500), 1); // 至少 1
    }

    #[test]
    fn heal_amount_rounds() {
        assert_eq!(heal_amount(20, 120), 24);
        assert_eq!(heal_amount(15, 110), 17); // 16.5
    }
}
