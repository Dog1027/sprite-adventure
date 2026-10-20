//! 商店畫面：用金幣購買捕捉球。其他道具尚未實作。

use super::{BALL_PRICE, Run, ui};
use crate::palette;
use crate::text;
use macroquad::prelude::*;

const CARD: Rect = Rect {
    x: 40.0,
    y: 110.0,
    w: 260.0,
    h: 220.0,
};

/// 畫出商店並處理購買；按下離開時回傳 true
pub fn frame(run: &mut Run) -> bool {
    text::draw("商店", 40.0, 50.0, 36.0, palette::TEXT);
    ui::draw_status(run);

    draw_rectangle(CARD.x, CARD.y, CARD.w, CARD.h, palette::PANEL);
    draw_rectangle_lines(CARD.x, CARD.y, CARD.w, CARD.h, 2.0, palette::PANEL_BORDER);
    text::draw("捕捉球", CARD.x + 16.0, CARD.y + 48.0, 36.0, palette::TEXT);
    text::draw(
        "在捕捉節點用來",
        CARD.x + 16.0,
        CARD.y + 90.0,
        24.0,
        palette::TEXT_DIM,
    );
    text::draw(
        "捕捉一隻精靈",
        CARD.x + 16.0,
        CARD.y + 118.0,
        24.0,
        palette::TEXT_DIM,
    );
    text::draw(
        &format!("{BALL_PRICE} 金幣"),
        CARD.x + 16.0,
        CARD.y + 158.0,
        24.0,
        palette::HIGHLIGHT,
    );

    let afford = run.gold >= BALL_PRICE;
    let buy = Rect::new(CARD.x + 16.0, CARD.y + CARD.h - 52.0, CARD.w - 32.0, 40.0);
    if ui::button(buy, "購買", afford, false) {
        run.buy_ball();
    }
    if !afford {
        text::draw(
            "金幣不夠",
            CARD.x,
            CARD.y + CARD.h + 32.0,
            24.0,
            palette::WARNING,
        );
    }

    text::draw("其他道具尚未實作", 40.0, 470.0, 24.0, palette::TEXT_MUTED);
    ui::button(Rect::new(600.0, 520.0, 160.0, 50.0), "離開", true, false)
}
