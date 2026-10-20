//! 休息畫面：選一隻精靈，再選要強化 HP、攻擊或防禦。規則見 docs/DESIGN.md「精靈」。

use super::Run;
use super::ui::{self, BOTTOM_BUTTON, HEADER_H, MARGIN, Style};
use crate::palette;
use crate::screen::W;
use crate::sprite::growth::Boost;
use crate::text;
use macroquad::prelude::*;

const GRID_TOP: f32 = HEADER_H + 48.0;
const CARD_H: f32 = 52.0;
const CARD_GAP: f32 = 9.0;
/// 隊伍沒有上限，一頁兩排各 3 隻，其餘用左右箭頭翻頁
const CARDS_PER_ROW: usize = 3;
const CARDS_PER_PAGE: usize = CARDS_PER_ROW * 2;
const PAGER_TOP: f32 = GRID_TOP + 2.0 * (CARD_H + CARD_GAP);
const PANEL_TOP: f32 = PAGER_TOP + 56.0;

/// 這一頁第 n 張卡片的位置
fn card_rect(n: usize) -> Rect {
    let w = (W - 2.0 * MARGIN - CARD_GAP * (CARDS_PER_ROW - 1) as f32) / CARDS_PER_ROW as f32;
    let (row, col) = (n / CARDS_PER_ROW, n % CARDS_PER_ROW);
    Rect::new(
        MARGIN + col as f32 * (w + CARD_GAP),
        GRID_TOP + row as f32 * (CARD_H + CARD_GAP),
        w,
        CARD_H,
    )
}

#[derive(Default)]
pub struct RestScreen {
    /// 選中的隊伍精靈
    pub(super) selected: Option<usize>,
    pub(super) page: usize,
    /// 已經強化：哪一隻、哪一項、提升多少
    pub(super) done: Option<(usize, Boost, u32)>,
}

impl RestScreen {
    /// 畫出休息畫面並處理強化；按下繼續或離開時回傳 true
    pub fn frame(&mut self, run: &mut Run) -> bool {
        ui::header("休息", Some(run));

        if let Some((i, boost, amount)) = self.done {
            let r = Rect::new(MARGIN, HEADER_H + 120.0, W - 2.0 * MARGIN, 160.0);
            ui::card(r);
            text::centered(
                "強化完成",
                vec2(W / 2.0, r.y + 40.0),
                24.0,
                palette::TEXT_DIM,
            );
            text::centered(
                &run.party[i].name,
                vec2(W / 2.0, r.y + 84.0),
                36.0,
                palette::TEXT,
            );
            text::centered(
                &format!("{} +{}", boost.label(), amount),
                vec2(W / 2.0, r.y + 126.0),
                36.0,
                palette::SUCCESS,
            );
            return ui::button(BOTTOM_BUTTON, "繼續", Style::Primary, true);
        }

        text::draw(
            "選一隻精靈，強化一項能力值",
            MARGIN,
            HEADER_H + 30.0,
            18.0,
            palette::TEXT_DIM,
        );
        self.draw_party(run);
        match self.selected {
            Some(i) => {
                if let Some(boost) = draw_boosts(run, i) {
                    let amount = run.rest(i, boost);
                    self.done = Some((i, boost, amount));
                }
            }
            None => text::centered(
                "先選一隻精靈",
                vec2(W / 2.0, PANEL_TOP + 60.0),
                24.0,
                palette::TEXT_MUTED,
            ),
        }
        ui::button(BOTTOM_BUTTON, "不休息，離開", Style::Normal, true)
    }

    fn draw_party(&mut self, run: &Run) {
        let first = self.page * CARDS_PER_PAGE;
        for i in first..run.party.len().min(first + CARDS_PER_PAGE) {
            let style = if self.selected == Some(i) {
                Style::Selected
            } else {
                Style::Normal
            };
            if ui::button(card_rect(i - first), &run.party[i].name, style, true) {
                self.selected = Some(i);
            }
        }

        let pages = run.party.len().div_ceil(CARDS_PER_PAGE).max(1);
        if pages > 1 {
            self.page = ui::pager(
                Rect::new(MARGIN, PAGER_TOP, 56.0, 44.0),
                Rect::new(W - MARGIN - 56.0, PAGER_TOP, 56.0, 44.0),
                self.page,
                pages,
            );
            text::centered(
                &format!("{} / {}", self.page + 1, pages),
                vec2(W / 2.0, PAGER_TOP + 22.0),
                18.0,
                palette::TEXT_DIM,
            );
        }
    }
}

/// 畫出選中精靈的三個強化選項，顯示強化前後的數值；點擊時回傳選的項目
fn draw_boosts(run: &Run, i: usize) -> Option<Boost> {
    let sprite = &run.party[i];
    text::draw(&sprite.name, MARGIN, PANEL_TOP + 30.0, 36.0, palette::TEXT);
    let mut chosen = None;
    for (n, boost) in Boost::ALL.into_iter().enumerate() {
        let value = boost.value(&sprite.stats);
        let amount = sprite.enhance_amount(boost);
        let rect = Rect::new(
            MARGIN,
            PANEL_TOP + 52.0 + n as f32 * 66.0,
            W - 2.0 * MARGIN,
            56.0,
        );
        if ui::button(rect, "", Style::Normal, true) {
            chosen = Some(boost);
        }
        // 按鈕上的文字分左右兩邊：項目名稱、強化前後
        let y = rect.center().y + 9.0;
        text::draw(boost.label(), rect.x + 20.0, y, 24.0, palette::TEXT);
        let after = format!("{value} → {}", value + amount);
        let plus = format!("+{amount}");
        let plus_w = text::measure(&plus, 24.0).width;
        text::right(&plus, rect.x + rect.w - 20.0, y, 24.0, palette::SUCCESS);
        text::right(
            &after,
            rect.x + rect.w - 36.0 - plus_w,
            y,
            24.0,
            palette::TEXT_DIM,
        );
    }
    text::draw(
        "節奏數值（攻擊間隔等）不能強化",
        MARGIN,
        PANEL_TOP + 52.0 + 3.0 * 66.0 + 20.0,
        18.0,
        palette::TEXT_MUTED,
    );
    chosen
}
