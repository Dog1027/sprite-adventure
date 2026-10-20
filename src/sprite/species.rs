//! 精靈種類資料，讀自 `data/species.toml`（見 docs/DESIGN.md「其他」）。
//!
//! 資料檔在編譯時嵌入程式，原生版與網頁版都不需要另外讀檔。

use std::collections::BTreeMap;
use std::sync::OnceLock;

use super::{Sprite, Stats};

const DATA: &str = include_str!("../../data/species.toml");

/// 解析並檢查資料檔，回傳「名稱 → 數值」
fn parse(text: &str) -> Result<BTreeMap<String, Stats>, String> {
    let table: BTreeMap<String, Stats> =
        toml::from_str(text).map_err(|e| format!("精靈資料格式錯誤：{e}"))?;
    for (name, s) in &table {
        for (field, value) in [
            ("hp", s.hp),
            ("interval", s.interval),
            ("start_turn", s.start_turn),
            ("skill_every", s.skill_every),
        ] {
            if value < 1 {
                return Err(format!("精靈 {name} 的 {field} 至少要是 1"));
            }
        }
    }
    Ok(table)
}

fn table() -> &'static BTreeMap<String, Stats> {
    static TABLE: OnceLock<BTreeMap<String, Stats>> = OnceLock::new();
    TABLE.get_or_init(|| parse(DATA).unwrap_or_else(|e| panic!("data/species.toml：{e}")))
}

/// 依名稱建立精靈
pub fn create(name: &str) -> Sprite {
    let stats = *table()
        .get(name)
        .unwrap_or_else(|| panic!("未知的精靈：{name}"));
    Sprite::new(name, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn rejects_zero_interval() {
        let text = "[X]\nhp = 1\nattack = 1\ndefense = 0\ninterval = 0\nstart_turn = 1\nskill_every = 1\n";
        assert!(parse(text).unwrap_err().contains("interval"));
    }

    #[test]
    fn rejects_unknown_and_missing_fields() {
        let typo = "[X]\nhp = 1\natack = 1\ndefense = 0\ninterval = 1\nstart_turn = 1\nskill_every = 1\n";
        assert!(parse(typo).is_err());
    }
}
