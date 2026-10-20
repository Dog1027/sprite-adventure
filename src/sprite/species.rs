//! 精靈種類資料，讀自 `data/species.toml`（見 docs/DESIGN.md「其他」）。
//!
//! 資料檔在編譯時嵌入程式，原生版與網頁版都不需要另外讀檔。

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::{Skill, Sprite, Stats};

const DATA: &str = include_str!("../../data/species.toml");

/// 資料檔中一種精靈的內容
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    hp: u32,
    attack: u32,
    defense: u32,
    interval: u32,
    start_turn: u32,
    skill_every: u32,
    skill: Skill,
    catch_rate: u32,
}

/// 一種精靈：建立用的範本與捕捉率
#[derive(Debug)]
struct Species {
    sprite: Sprite,
    catch_rate: u32,
}

/// 解析並檢查資料檔，回傳「名稱 → 精靈種類」
fn parse(text: &str) -> Result<BTreeMap<String, Species>, String> {
    let table: BTreeMap<String, Entry> =
        toml::from_str(text).map_err(|e| format!("精靈資料格式錯誤：{e}"))?;
    let mut sprites = BTreeMap::new();
    for (name, e) in table {
        for (field, value) in [
            ("hp", e.hp),
            ("interval", e.interval),
            ("start_turn", e.start_turn),
            ("skill_every", e.skill_every),
            ("skill.power", e.skill.power),
        ] {
            if value < 1 {
                return Err(format!("精靈 {name} 的 {field} 至少要是 1"));
            }
        }
        if e.catch_rate > 100 {
            return Err(format!("精靈 {name} 的 catch_rate 不能超過 100"));
        }
        let stats = Stats {
            hp: e.hp,
            attack: e.attack,
            defense: e.defense,
            interval: e.interval,
            start_turn: e.start_turn,
            skill_every: e.skill_every,
        };
        let sprite = Sprite::new(&name, stats).with_skill(e.skill);
        let species = Species {
            sprite,
            catch_rate: e.catch_rate,
        };
        sprites.insert(name, species);
    }
    Ok(sprites)
}

fn table() -> &'static BTreeMap<String, Species> {
    static TABLE: OnceLock<BTreeMap<String, Species>> = OnceLock::new();
    TABLE.get_or_init(|| parse(DATA).unwrap_or_else(|e| panic!("data/species.toml：{e}")))
}

fn get(name: &str) -> &'static Species {
    table()
        .get(name)
        .unwrap_or_else(|| panic!("未知的精靈：{name}"))
}

/// 依名稱建立精靈
pub fn create(name: &str) -> Sprite {
    get(name).sprite.clone()
}

/// 在捕捉節點的基本捕捉機率（%），會直接顯示給玩家。道具的影響之後再加
#[allow(dead_code)] // 捕捉節點尚未實作
pub fn catch_rate(name: &str) -> u32 {
    get(name).catch_rate
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sprite::skill::Range;

    const VALID: &str = "[X]\nhp = 1\nattack = 1\ndefense = 0\ninterval = 1\nstart_turn = 1\nskill_every = 1\n\
                         skill = { name = \"s\", range = \"row\", power = 80 }\ncatch_rate = 40\n";

    #[test]
    fn data_file_is_valid() {
        let t = parse(DATA).unwrap();
        assert!(!t.is_empty());
    }

    #[test]
    fn create_reads_data_file() {
        let s = create("A1");
        assert_eq!(s.name, "A1");
        assert_eq!(s.stats.hp, 120);
        assert_eq!(s.stats.skill_every, 3);
        assert_eq!(s.skill.range, Range::Row);
    }

    #[test]
    fn parses_skill() {
        let t = parse(VALID).unwrap();
        let skill = &t["X"].sprite.skill;
        assert_eq!((skill.range, skill.power), (Range::Row, 80));
    }

    #[test]
    fn reads_catch_rate() {
        assert_eq!(parse(VALID).unwrap()["X"].catch_rate, 40);
        assert!(catch_rate("E1") <= 100);
    }

    #[test]
    fn rejects_catch_rate_over_100() {
        let text = VALID.replace("catch_rate = 40", "catch_rate = 101");
        assert!(parse(&text).unwrap_err().contains("catch_rate"));
    }

    #[test]
    fn rejects_zero_interval() {
        let text = VALID.replace("interval = 1", "interval = 0");
        assert!(parse(&text).unwrap_err().contains("interval"));
    }

    #[test]
    fn rejects_typos() {
        assert!(parse(&VALID.replace("attack", "atack")).is_err());
        assert!(parse(&VALID.replace("\"row\"", "\"rows\"")).is_err());
    }

    #[test]
    fn rejects_missing_skill() {
        let text = VALID.lines().take(7).collect::<Vec<_>>().join("\n");
        assert!(parse(&text).is_err());
    }
}
