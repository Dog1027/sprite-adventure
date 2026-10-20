//! 商店畫面：用金幣購買捕捉球。其他道具尚未實作。

use super::ui::{self, BOTTOM_BUTTON, HEADER_H, MARGIN, Style};
use super::{BALL_PRICE, Run};
use crate::palette;
use crate::screen::W;
use crate::text;
use macroquad::prelude::*;

const CARD: Rect = Rect {
    x: MARGIN,
    y: HEADER_H + 24.0,
    w: W - 2.0 * MARGIN,
    h: 148.0,
};

/// 畫出商店並處理購買；按下離開時回傳 true
pub fn frame(run: &mut Run) -> bool {
    ui::header("商店", Some(run));

    ui::card(CARD);
    ui::ball_icon(vec2(CARD.x + 48.0, CARD.y + 52.0), 26.0);
    let left = CARD.x + 92.0;
    text::draw("捕捉球", left, CARD.y + 44.0, 24.0, palette::TEXT);
    text::draw(
        "在捕捉節點捕捉精靈",
        left,
        CARD.y + 72.0,
        18.0,
        palette::TEXT_DIM,
    );
    ui::coin_icon(vec2(left + 10.0, CARD.y + 116.0), 9.0);
    text::draw(
        &BALL_PRICE.to_string(),
        left + 26.0,
        CARD.y + 125.0,
        24.0,
        palette::HIGHLIGHT,
    );

    let afford = run.gold >= BALL_PRICE;
    let buy = Rect::new(CARD.x + CARD.w - 116.0, CARD.y + CARD.h - 56.0, 100.0, 44.0);
    if ui::button(buy, "購買", Style::Primary, afford) {
        run.buy_ball();
    }
    if !afford {
        text::right(
            "金幣不夠",
            CARD.x + CARD.w - 16.0,
            CARD.y + 30.0,
            18.0,
            palette::WARNING,
        );
    }

    text::centered(
        "其他道具尚未實作",
        vec2(W / 2.0, CARD.y + CARD.h + 40.0),
        18.0,
        palette::TEXT_MUTED,
    );
    ui::button(BOTTOM_BUTTON, "離開", Style::Normal, true)
}
