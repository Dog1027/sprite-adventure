//! 捕捉畫面：出現幾隻野生精靈，顯示能力值與捕捉率，選一隻加入隊伍或略過。

use super::{Run, ui};
use crate::sprite::skill::Range;
use crate::sprite::species;
use crate::text;
use macroquad::prelude::*;

const CARD_TOP: f32 = 110.0;
const CARD_W: f32 = 220.0;
const CARD_H: f32 = 370.0;
const CARD_GAP: f32 = 20.0;

pub enum Choice {
    Catch(&'static str),
    Skip,
}

fn range_label(range: Range) -> &'static str {
    match range {
        Range::Single => "單體",
        Range::Row => "貫穿",
        Range::Front => "前排",
        Range::Snipe => "狙擊",
        Range::All => "全體",
        Range::Heal => "治療",
    }
}

/// 畫出捕捉畫面；玩家選擇後回傳
pub fn frame(run: &Run, offers: &[&'static str]) -> Option<Choice> {
    text::draw("捕捉", 40.0, 50.0, 36.0, WHITE);
    text::draw(
        "用一個捕捉球，選一隻加入隊伍（目前必定成功）",
        40.0,
        90.0,
        24.0,
        LIGHTGRAY,
    );
    let no_balls = run.balls == 0;
    ui::draw_status(run);

    let mut choice = None;
    for (i, &name) in offers.iter().enumerate() {
        let x = 40.0 + i as f32 * (CARD_W + CARD_GAP);
        draw_rectangle(
            x,
            CARD_TOP,
            CARD_W,
            CARD_H,
            Color::new(0.22, 0.22, 0.26, 1.0),
        );
        draw_rectangle_lines(x, CARD_TOP, CARD_W, CARD_H, 2.0, GRAY);

        let sprite = species::create(name);
        let s = &sprite.stats;
        let skill = &sprite.skill;
        text::draw(&sprite.name, x + 16.0, CARD_TOP + 48.0, 36.0, WHITE);
        let lines = [
            format!("HP {}", s.hp),
            format!("攻擊 {} 防禦 {}", s.attack, s.defense),
            format!("每 {} 回合攻擊", s.interval),
            format!("第 {} 回合開始", s.start_turn),
            format!("技能「{}」", skill.name),
            format!("{} {}%", range_label(skill.range), skill.power),
            format!("每 {} 次攻擊一次", s.skill_every),
        ];
        for (n, line) in lines.iter().enumerate() {
            text::draw(
                line,
                x + 16.0,
                CARD_TOP + 90.0 + n as f32 * 28.0,
                24.0,
                LIGHTGRAY,
            );
        }
        let rate = format!("捕捉率 {}%", species::catch_rate(name));
        text::draw(&rate, x + 16.0, CARD_TOP + 300.0, 24.0, GOLD);

        let button = Rect::new(x + 16.0, CARD_TOP + CARD_H - 56.0, CARD_W - 32.0, 44.0);
        if ui::button(button, "捕捉", !no_balls, false) {
            choice = Some(Choice::Catch(name));
        }
    }

    if no_balls {
        text::draw("沒有捕捉球了", 40.0, 540.0, 24.0, ORANGE);
    }
    if ui::button(Rect::new(600.0, 520.0, 160.0, 50.0), "略過", true, false) {
        choice = Some(Choice::Skip);
    }
    choice
}
