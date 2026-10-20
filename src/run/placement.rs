//! 陣型選擇與擺位畫面：看得到敵方陣容，選陣型、把精靈放進開放的格子，預覽線顯示第一個攻擊目標。

use super::Run;
use super::ui::{self, BOTTOM_BUTTON, MARGIN, Style};
use crate::battle::formation::{MIN_UNITS, bonus_label};
use crate::battle::view::{self, BOARD_BOTTOM};
use crate::battle::{Side, Unit};
use crate::gfx;
use crate::palette;
use crate::screen::W;
use crate::sprite::Sprite;
use crate::text;
use macroquad::prelude::*;

const TABS_TOP: f32 = 72.0;
const CARD_TOP: f32 = BOARD_BOTTOM + 10.0;
const CARD_W: f32 = 80.0;
const CARD_H: f32 = 52.0;
const CARD_GAP: f32 = 8.0;
const ARROW_W: f32 = 32.0;
/// 隊伍沒有上限，一頁顯示幾隻，其餘用左右箭頭翻頁
const CARDS_PER_PAGE: usize = 4;
const CARDS_LEFT: f32 = (W - CARDS_PER_PAGE as f32 * (CARD_W + CARD_GAP) + CARD_GAP) / 2.0;

/// 這一頁第 n 張卡片的位置
fn card_rect(n: usize) -> Rect {
    Rect::new(
        CARDS_LEFT + n as f32 * (CARD_W + CARD_GAP),
        CARD_TOP,
        CARD_W,
        CARD_H,
    )
}

pub struct PlacementScreen {
    pub enemies: Vec<Unit>,
    formation: usize,
    /// 每一格放的是隊伍中第幾隻精靈
    slots: Vec<Option<usize>>,
    /// 選中的隊伍精靈，下一次點格子時放進去
    selected: Option<usize>,
    /// 隊伍卡片目前在第幾頁
    page: usize,
}

impl PlacementScreen {
    pub fn new(run: &Run, enemies: Vec<Unit>) -> Self {
        let mut screen = Self {
            enemies,
            formation: 0,
            slots: Vec::new(),
            selected: None,
            page: 0,
        };
        screen.auto_fill(run);
        screen
    }

    /// 依隊伍順序填滿陣型的格子
    fn auto_fill(&mut self, run: &Run) {
        let n = run.formations[self.formation].slots.len();
        self.slots = (0..n).map(|i| (i < run.party.len()).then_some(i)).collect();
        self.selected = None;
    }

    fn placement<'a>(&self, run: &'a Run) -> Vec<Option<&'a Sprite>> {
        self.slots
            .iter()
            .map(|s| s.map(|i| &run.party[i]))
            .collect()
    }

    /// 選中隊伍中第 i 隻（開發用截圖也會用到）
    pub(super) fn select(&mut self, i: usize) {
        self.selected = Some(i);
    }

    /// 畫出畫面並處理點擊；按下開戰時回傳擺好的我方精靈
    pub fn frame(&mut self, run: &Run) -> Option<Vec<Unit>> {
        ui::header("布陣", Some(run));
        self.draw_tabs(run);

        let formation = &run.formations[self.formation];
        view::draw_boards();
        // 開放的格子；空格的陣型加成寫在地上
        for (i, &pos) in formation.slots.iter().enumerate() {
            view::draw_slot(
                Side::Ally,
                pos,
                palette::OPEN_SLOT,
                palette::OPEN_SLOT_BORDER,
            );
            let label = bonus_label(&formation.slot_percents(i));
            if self.slots[i].is_none() && !label.is_empty() {
                text::centered(
                    &label,
                    view::cell_center(Side::Ally, pos),
                    12.0,
                    palette::HIGHLIGHT,
                );
            }
        }

        // 我方精靈（含陣型加成，預覽才準），由後往前畫
        let placement = self.placement(run);
        let mut units = formation.place(&placement, Side::Ally).unwrap_or_default();
        units.extend(self.enemies.iter().cloned());
        for i in view::draw_order(&units) {
            view::draw_unit(&units[i]);
        }
        // 站在有加成的格子上：立牌右上角一個金色圓點（選中時下方會寫出加成）
        for (i, &pos) in formation.slots.iter().enumerate() {
            if self.slots[i].is_some() && !formation.slot_percents(i).is_empty() {
                let r = view::unit_rect(Side::Ally, pos);
                draw_circle(r.x + r.w - 3.0, r.y + 3.0, 5.0, palette::HIGHLIGHT);
            }
        }
        view::draw_preview(&units);
        if let Some(i) = self.selected
            && let Some(slot) = self.slots.iter().position(|&s| s == Some(i))
        {
            let r = view::unit_rect(Side::Ally, formation.slots[slot]);
            gfx::rounded_rect_lines(r, 8.0, 3.0, palette::BUTTON_ACTIVE);
        }

        // 點格子：有選中的精靈就放進去，否則拿起格子裡的精靈
        if ui::clicked()
            && let Some(pos) = view::cell_at(Side::Ally, ui::mouse())
            && let Some(slot) = formation.slots.iter().position(|&p| p == pos)
        {
            match self.selected.take() {
                Some(member) => {
                    for s in &mut self.slots {
                        if *s == Some(member) {
                            *s = None;
                        }
                    }
                    self.slots[slot] = Some(member);
                }
                None => self.selected = self.slots[slot].take(),
            }
        }

        self.draw_party(run);

        let placed = self.slots.iter().flatten().count();
        let ready = placed >= MIN_UNITS;
        let label = if ready {
            format!("開戰（{placed} 隻上場）")
        } else {
            format!("至少擺 {MIN_UNITS} 隻")
        };
        if ui::button(BOTTOM_BUTTON, &label, Style::Primary, ready) {
            return formation.place(&placement, Side::Ally).ok();
        }
        None
    }

    /// 上方的陣型分頁
    fn draw_tabs(&mut self, run: &Run) {
        let n = run.formations.len() as f32;
        let gap = 8.0;
        let w = (W - 2.0 * MARGIN - gap * (n - 1.0)) / n;
        for (i, f) in run.formations.iter().enumerate() {
            let rect = Rect::new(MARGIN + i as f32 * (w + gap), TABS_TOP, w, 44.0);
            let label = format!("{} {}格", f.name, f.slots.len());
            let style = if i == self.formation {
                Style::Selected
            } else {
                Style::Normal
            };
            if ui::button(rect, &label, style, true) && i != self.formation {
                self.formation = i;
                self.auto_fill(run);
            }
        }
    }

    fn draw_party(&mut self, run: &Run) {
        let pages = run.party.len().div_ceil(CARDS_PER_PAGE).max(1);
        let arrow_y = CARD_TOP + (CARD_H - 44.0) / 2.0;
        self.page = ui::pager(
            Rect::new(MARGIN - 4.0, arrow_y, ARROW_W, 44.0),
            Rect::new(W - MARGIN - ARROW_W + 4.0, arrow_y, ARROW_W, 44.0),
            self.page,
            pages,
        );

        let first = self.page * CARDS_PER_PAGE;
        for i in first..run.party.len().min(first + CARDS_PER_PAGE) {
            let rect = card_rect(i - first);
            let style = if self.selected == Some(i) {
                Style::Selected
            } else {
                Style::Normal
            };
            if ui::button(rect, &run.party[i].name, style, true) {
                self.selected = if self.selected == Some(i) {
                    None
                } else {
                    Some(i)
                };
            }
            // 已經上場的精靈，右上角一個綠點
            if self.slots.contains(&Some(i)) {
                draw_circle(rect.x + rect.w - 9.0, rect.y + 9.0, 4.0, palette::SUCCESS);
            }
        }

        let info_y = CARD_TOP + CARD_H + 26.0;
        match self.selected {
            Some(i) => {
                let sprite = &run.party[i];
                let s = &sprite.stats;
                let line1 = format!(
                    "{}　HP {}　攻擊 {}　防禦 {}",
                    sprite.name, s.hp, s.attack, s.defense
                );
                let line2 = format!(
                    "每 {} 回合攻擊（第 {} 回合起）・技能每 {} 次",
                    s.interval, s.start_turn, s.skill_every
                );
                text::draw(&line1, MARGIN, info_y, 18.0, palette::TEXT);
                // 擺在有加成的格子上時，右邊寫出這一格的加成
                let formation = &run.formations[self.formation];
                if let Some(slot) = self.slots.iter().position(|&s| s == Some(i)) {
                    let bonus = bonus_label(&formation.slot_percents(slot));
                    if !bonus.is_empty() {
                        text::right(&bonus, W - MARGIN, info_y, 18.0, palette::HIGHLIGHT);
                    }
                }
                text::draw(&line2, MARGIN, info_y + 22.0, 18.0, palette::TEXT_DIM);
            }
            None => {
                text::draw(
                    "點隊伍中的精靈，再點綠色格子放上去",
                    MARGIN,
                    info_y,
                    18.0,
                    palette::TEXT_DIM,
                );
                text::draw(
                    "點格子裡的精靈可以拿起來",
                    MARGIN,
                    info_y + 22.0,
                    18.0,
                    palette::TEXT_MUTED,
                );
            }
        }
    }
}
