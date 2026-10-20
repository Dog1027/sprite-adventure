//! 休息畫面：選一隻精靈，再選要強化 HP、攻擊或防禦。規則見 docs/DESIGN.md「精靈」。

use super::{Run, ui};
use crate::sprite::growth::Boost;
use crate::text;
use macroquad::prelude::*;

const CARD_TOP: f32 = 110.0;
const CARD_W: f32 = 120.0;
const CARD_H: f32 = 50.0;
const CARD_GAP: f32 = 12.0;
const CARDS_LEFT: f32 = 76.0;
/// 隊伍沒有上限，一頁兩排各 5 隻，其餘用左右箭頭翻頁
const CARDS_PER_ROW: usize = 5;
const CARDS_PER_PAGE: usize = CARDS_PER_ROW * 2;
const PANEL_TOP: f32 = 260.0;

/// 這一頁第 n 張卡片的位置
fn card_rect(n: usize) -> Rect {
    let (row, col) = (n / CARDS_PER_ROW, n % CARDS_PER_ROW);
    Rect::new(
        CARDS_LEFT + col as f32 * (CARD_W + CARD_GAP),
        CARD_TOP + row as f32 * (CARD_H + CARD_GAP),
        CARD_W,
        CARD_H,
    )
}

#[derive(Default)]
pub struct RestScreen {
    /// 選中的隊伍精靈
    selected: Option<usize>,
    page: usize,
    /// 已經強化：哪一隻、哪一項、提升多少
    done: Option<(usize, Boost, u32)>,
}

impl RestScreen {
    /// 畫出休息畫面並處理強化；按下繼續或離開時回傳 true
    pub fn frame(&mut self, run: &mut Run) -> bool {
        text::draw("休息", 40.0, 50.0, 36.0, WHITE);
        ui::draw_status(run);

        if let Some((i, boost, amount)) = self.done {
            let line = format!("{} 的{} +{}", run.party[i].name, boost.label(), amount);
            let center = vec2(400.0, 260.0);
            text::centered(&line, center, 36.0, GREEN);
            text::centered("點擊繼續", center + vec2(0.0, 80.0), 24.0, LIGHTGRAY);
            return ui::clicked();
        }

        text::draw("選一隻精靈強化一項能力值", 40.0, 90.0, 24.0, LIGHTGRAY);
        self.draw_party(run);
        if let Some(i) = self.selected
            && let Some(boost) = draw_boosts(run, i)
        {
            let amount = run.rest(i, boost);
            self.done = Some((i, boost, amount));
        }
        ui::button(Rect::new(600.0, 520.0, 160.0, 50.0), "離開", true, false)
    }

    fn draw_party(&mut self, run: &Run) {
        let pages = run.party.len().div_ceil(CARDS_PER_PAGE).max(1);
        self.page = self.page.min(pages - 1);
        let arrow_y = CARD_TOP + CARD_H + (CARD_GAP - 40.0) / 2.0;
        if ui::button(
            Rect::new(40.0, arrow_y, 28.0, 40.0),
            "<",
            self.page > 0,
            false,
        ) {
            self.page -= 1;
        }
        let right = CARDS_LEFT + CARDS_PER_ROW as f32 * (CARD_W + CARD_GAP);
        if ui::button(
            Rect::new(right, arrow_y, 28.0, 40.0),
            ">",
            self.page + 1 < pages,
            false,
        ) {
            self.page += 1;
        }

        let first = self.page * CARDS_PER_PAGE;
        for i in first..run.party.len().min(first + CARDS_PER_PAGE) {
            let rect = card_rect(i - first);
            if ui::button(rect, &run.party[i].name, true, self.selected == Some(i)) {
                self.selected = Some(i);
            }
        }
    }
}

/// 畫出選中精靈的三個強化選項，顯示強化前後的數值；點擊時回傳選的項目
fn draw_boosts(run: &Run, i: usize) -> Option<Boost> {
    let sprite = &run.party[i];
    text::draw(&sprite.name, 40.0, PANEL_TOP, 36.0, WHITE);
    let mut chosen = None;
    for (n, boost) in Boost::ALL.into_iter().enumerate() {
        let value = boost.value(&sprite.stats);
        let amount = sprite.enhance_amount(boost);
        let label = format!("{} {} → {}", boost.label(), value, value + amount);
        let rect = Rect::new(40.0 + n as f32 * 230.0, PANEL_TOP + 30.0, 210.0, 60.0);
        if ui::button(rect, &label, true, false) {
            chosen = Some(boost);
        }
    }
    text::draw("節奏數值不能強化", 40.0, PANEL_TOP + 130.0, 24.0, GRAY);
    chosen
}
