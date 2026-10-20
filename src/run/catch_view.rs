//! 捕捉畫面：出現幾隻野生精靈，顯示能力值與捕捉率，選一隻加入隊伍或略過。

use super::Run;
use super::ui::{self, BOTTOM_BUTTON, HEADER_H, MARGIN, Style};
use crate::gfx;
use crate::palette;
use crate::screen::W;
use crate::sprite::species;
use crate::text;
use macroquad::prelude::*;

const CARD_TOP: f32 = HEADER_H + 48.0;
const CARD_H: f32 = 176.0;
const CARD_GAP: f32 = 12.0;

pub enum Choice {
    Catch(&'static str),
    Skip,
}

/// 畫出捕捉畫面；玩家選擇後回傳
pub fn frame(run: &Run, offers: &[&'static str]) -> Option<Choice> {
    ui::header("捕捉", Some(run));
    let no_balls = run.balls == 0;
    let hint = if no_balls {
        "沒有捕捉球了，可以在商店購買"
    } else {
        "用 1 顆捕捉球帶走一隻（目前必定成功）"
    };
    let hint_color = if no_balls {
        palette::WARNING
    } else {
        palette::TEXT_DIM
    };
    text::draw(hint, MARGIN, HEADER_H + 30.0, 18.0, hint_color);

    let mut choice = None;
    for (i, &id) in offers.iter().enumerate() {
        let r = Rect::new(
            MARGIN,
            CARD_TOP + i as f32 * (CARD_H + CARD_GAP),
            W - 2.0 * MARGIN,
            CARD_H,
        );
        if draw_offer(r, id, !no_balls) {
            choice = Some(Choice::Catch(id));
        }
    }

    if ui::button(BOTTOM_BUTTON, "略過", Style::Normal, true) {
        choice = Some(Choice::Skip);
    }
    choice
}

/// 一隻精靈的卡片；按下捕捉時回傳 true
fn draw_offer(r: Rect, id: &'static str, can_catch: bool) -> bool {
    ui::card(r);
    let (rarity, rarity_color) = ui::rarity_label(species::rarity(id));
    // 左側的稀有度色條
    gfx::rounded_rect(Rect::new(r.x, r.y, 6.0, r.h), 3.0, rarity_color);

    let sprite = species::create(id);
    let s = &sprite.stats;
    let skill = &sprite.skill;
    let left = r.x + 20.0;
    text::draw(&sprite.name, left, r.y + 44.0, 36.0, palette::TEXT);
    let name_w = text::measure(&sprite.name, 36.0).width;
    ui::chip(rarity, left + name_w + 12.0, r.y + 32.0, rarity_color);
    text::right(
        &format!("捕捉率 {}%", species::catch_rate(id)),
        r.x + r.w - 16.0,
        r.y + 40.0,
        18.0,
        palette::HIGHLIGHT,
    );

    let lines = [
        (
            format!("HP {}　攻擊 {}　防禦 {}", s.hp, s.attack, s.defense),
            palette::TEXT,
        ),
        (
            format!("每 {} 回合攻擊（第 {} 回合起）", s.interval, s.start_turn),
            palette::TEXT_DIM,
        ),
        (
            format!(
                "技能「{}」{} {}%",
                skill.name,
                ui::range_label(skill.range),
                skill.power
            ),
            palette::SKILL,
        ),
        (
            format!("每 {} 次攻擊放一次技能", s.skill_every),
            palette::TEXT_DIM,
        ),
    ];
    for (n, (line, color)) in lines.iter().enumerate() {
        text::draw(line, left, r.y + 78.0 + n as f32 * 25.0, 18.0, *color);
    }

    let button = Rect::new(r.x + r.w - 116.0, r.y + r.h - 60.0, 100.0, 44.0);
    ui::button(button, "捕捉", Style::Primary, can_catch)
}
