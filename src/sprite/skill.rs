//! 技能資料：名稱、範圍、倍率。規則見 docs/DESIGN.md「技能」。
//!
//! 這裡只定義技能「是什麼」；技能效果的執行在戰鬥模組。

use serde::Deserialize;

/// 技能範圍。「敵方」的前後以敵方棋盤為準，depth 0 為最前方。
#[allow(dead_code)] // 技能效果由戰鬥模組執行，尚未實作
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Range {
    /// 單體：打普攻的目標
    Single,
    /// 貫穿：普攻目標所在那一列的所有敵人
    Row,
    /// 前排：最前面有敵人的那一行（同一個 depth）的所有敵人
    Front,
    /// 狙擊：最後方的敵人；同樣後面時比剩餘血量比例，再平手隨機
    Snipe,
    /// 全體：所有敵人
    All,
    /// 治療：剩餘血量比例最低的我方（含自己）
    Heal,
}

/// 一隻精靈的技能
#[allow(dead_code)] // 技能效果由戰鬥模組執行，尚未實作
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Skill {
    pub name: String,
    pub range: Range,
    /// 倍率（%）。傷害技能：以「攻擊 × 倍率」代入傷害公式；治療：治療量 = 攻擊 × 倍率
    pub power: u32,
}

impl Skill {
    /// 預設技能：等同普攻，給沒有指定技能的精靈（例如測試用）
    pub fn basic() -> Self {
        Self {
            name: "普攻".to_owned(),
            range: Range::Single,
            power: 100,
        }
    }
}
