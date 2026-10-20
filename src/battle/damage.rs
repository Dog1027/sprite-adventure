/// 傷害計算集中在這裡：攻擊 × 100 ÷ (100 + 防禦)，四捨五入，至少 1。
/// 之後的屬性倍率也在這裡乘上去；陣型加成在擺位時已套用到能力值（見 formation.rs）。
pub fn damage(attack: u32, defense: u32) -> u32 {
    let den = 100 + defense as u64;
    let rounded = (attack as u64 * 200 + den) / (den * 2);
    rounded.max(1) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damage_formula() {
        assert_eq!(damage(100, 0), 100);
        assert_eq!(damage(100, 100), 50);
        assert_eq!(damage(10, 50), 7); // 6.67
        assert_eq!(damage(3, 100), 2); // 1.5 四捨五入
        assert_eq!(damage(1, 1000), 1); // 至少 1
        assert_eq!(damage(0, 0), 1);
    }
}
