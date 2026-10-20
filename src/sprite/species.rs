//! 精靈種類資料。暫時寫死在程式中，之後改讀資料檔（見 docs/DESIGN.md「其他」）。

use super::{Sprite, Stats};

fn stats(
    hp: u32,
    attack: u32,
    defense: u32,
    interval: u32,
    start_turn: u32,
    skill_every: u32,
) -> Stats {
    Stats {
        hp,
        attack,
        defense,
        interval,
        start_turn,
        skill_every,
    }
}

/// 依名稱建立精靈；名稱與數值僅為示意
pub fn create(name: &str) -> Sprite {
    let s = match name {
        "A1" => stats(120, 30, 20, 1, 1, 3),
        "A2" => stats(80, 50, 5, 2, 1, 2),
        "A3" => stats(150, 20, 40, 1, 2, 4),
        "A4" => stats(60, 70, 0, 3, 3, 2),
        "E1" => stats(100, 35, 10, 1, 1, 3),
        "E2" => stats(140, 25, 30, 1, 1, 3),
        "E3" => stats(70, 60, 0, 2, 2, 2),
        _ => panic!("未知的精靈：{name}"),
    };
    Sprite::new(name, s)
}
