//! 精靈種類資料，讀自 `data/species.toml`（見 docs/DESIGN.md「其他」）。
//!
//! 資料檔在編譯時嵌入程式，原生版與網頁版都不需要另外讀檔。

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::{Skill, Sprite, Stats};

const DATA: &str = include_str!("../../data/species.toml");

/// 稀有度，影響在捕捉節點與敵方出現的機率（見 `generate`）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
}

/// 資料檔中一種精靈的內容
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    name: String,
    rarity: Rarity,
    #[serde(default)]
    starter: bool,
    hp: u32,
    attack: u32,
    defense: u32,
    interval: u32,
    start_turn: u32,
    skill_every: u32,
    skill: Skill,
    catch_rate: u32,
}

/// 一種精靈：建立用的範本、稀有度與捕捉率
#[derive(Debug)]
pub(super) struct Species {
    pub sprite: Sprite,
    pub rarity: Rarity,
    pub starter: bool,
    pub catch_rate: u32,
}

/// 解析並檢查資料檔，回傳「代號 → 精靈種類」
fn parse(text: &str) -> Result<BTreeMap<String, Species>, String> {
    let table: BTreeMap<String, Entry> =
        toml::from_str(text).map_err(|e| format!("精靈資料格式錯誤：{e}"))?;
    let mut all = BTreeMap::new();
    for (id, e) in table {
        for (field, value) in [
            ("hp", e.hp),
            ("interval", e.interval),
            ("start_turn", e.start_turn),
            ("skill_every", e.skill_every),
            ("skill.power", e.skill.power),
        ] {
            if value < 1 {
                return Err(format!("精靈 {id} 的 {field} 至少要是 1"));
            }
        }
        if e.catch_rate > 100 {
            return Err(format!("精靈 {id} 的 catch_rate 不能超過 100"));
        }
        if e.name.is_empty() {
            return Err(format!("精靈 {id} 沒有 name"));
        }
        let stats = Stats {
            hp: e.hp,
            attack: e.attack,
            defense: e.defense,
            interval: e.interval,
            start_turn: e.start_turn,
            skill_every: e.skill_every,
        };
        let species = Species {
            sprite: Sprite::new(&e.name, stats).with_skill(e.skill),
            rarity: e.rarity,
            starter: e.starter,
            catch_rate: e.catch_rate,
        };
        all.insert(id, species);
    }
    let starters = all.values().filter(|s| s.starter).count();
    if starters != 1 {
        return Err(format!("starter = true 要剛好一種，目前有 {starters} 種"));
    }
    Ok(all)
}

fn table() -> &'static BTreeMap<String, Species> {
    static TABLE: OnceLock<BTreeMap<String, Species>> = OnceLock::new();
    TABLE.get_or_init(|| parse(DATA).unwrap_or_else(|e| panic!("data/species.toml：{e}")))
}

fn get(id: &str) -> &'static Species {
    table()
        .get(id)
        .unwrap_or_else(|| panic!("未知的精靈：{id}"))
}

/// 所有精靈種類（依代號排序，順序固定，方便重現亂數結果）
pub(super) fn all() -> impl Iterator<Item = (&'static str, &'static Species)> {
    table().iter().map(|(id, s)| (id.as_str(), s))
}

/// 依代號建立精靈
pub fn create(id: &str) -> Sprite {
    get(id).sprite.clone()
}

/// 開局的精靈（資料檔中 `starter = true` 的那一種）
pub fn starter() -> Sprite {
    all()
        .find(|(_, s)| s.starter)
        .map(|(_, s)| s.sprite.clone())
        .expect("資料檔檢查過一定有開局精靈")
}

/// 在捕捉節點的基本捕捉機率（%），會直接顯示給玩家。道具的影響之後再加
pub fn catch_rate(id: &str) -> u32 {
    get(id).catch_rate
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sprite::skill::Range;

    const VALID: &str = "[x]\nname = \"甲\"\nrarity = \"common\"\nstarter = true\n\
                         hp = 1\nattack = 1\ndefense = 0\ninterval = 1\nstart_turn = 1\nskill_every = 1\n\
                         skill = { name = \"s\", range = \"row\", power = 80 }\ncatch_rate = 40\n";

    #[test]
    fn data_file_is_valid() {
        let t = parse(DATA).unwrap();
        assert!(t.len() >= 10);
    }

    #[test]
    fn create_uses_display_name() {
        let s = create("ember_fox");
        assert_eq!(s.name, "焰狐");
        assert_eq!(s.stats.hp, 100);
        assert_eq!(s.skill.range, Range::Single);
    }

    #[test]
    fn display_names_fit_a_cell() {
        for (id, s) in all() {
            assert!(s.sprite.name.chars().count() <= 2, "{id} 的名稱太長");
        }
    }

    #[test]
    fn has_one_starter() {
        assert_eq!(starter().name, "焰狐");
        let two = format!("{VALID}{}", VALID.replace("[x]", "[y]"));
        assert!(parse(&two).unwrap_err().contains("starter"));
        let none = VALID.replace("starter = true\n", "");
        assert!(parse(&none).unwrap_err().contains("starter"));
    }

    #[test]
    fn parses_skill_and_rarity() {
        let t = parse(VALID).unwrap();
        let x = &t["x"];
        assert_eq!(
            (x.sprite.skill.range, x.sprite.skill.power),
            (Range::Row, 80)
        );
        assert_eq!(x.rarity, Rarity::Common);
        assert_eq!(x.catch_rate, 40);
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
        assert!(parse(&VALID.replace("\"common\"", "\"normal\"")).is_err());
    }

    #[test]
    fn rejects_missing_fields() {
        assert!(parse(&VALID.replace("catch_rate = 40\n", "")).is_err());
        assert!(parse(&VALID.replace("name = \"甲\"\n", "")).is_err());
    }
}
