//! 陣型選擇與擺位畫面：看得到敵方陣容，選陣型、把精靈放進開放的格子，預覽線顯示第一個攻擊目標。

use super::Run;
use super::ui;
use crate::battle::formation::{MIN_UNITS, bonus_label};
use crate::battle::view::{self, CELL};
use crate::battle::{Side, Unit};
use crate::palette;
use crate::sprite::Sprite;
use crate::text;
use macroquad::prelude::*;

const CARD_TOP: f32 = 440.0;
const CARD_W: f32 = 80.0;
const CARD_H: f32 = 60.0;
const CARD_GAP: f32 = 8.0;
/// 隊伍沒有上限，一頁顯示幾隻，其餘用左右箭頭翻頁
const CARDS_PER_PAGE: usize = 5;
const CARDS_LEFT: f32 = 76.0;

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

    /// 畫出畫面並處理點擊；按下開戰時回傳擺好的我方精靈
    pub fn frame(&mut self, run: &Run) -> Option<Vec<Unit>> {
        text::draw("選擇陣型並擺放精靈", 40.0, 50.0, 36.0, palette::TEXT);

        for (i, f) in run.formations.iter().enumerate() {
            let rect = Rect::new(40.0 + i as f32 * 150.0, 70.0, 140.0, 40.0);
            let label = format!("{} {}格", f.name, f.slots.len());
            if ui::button(rect, &label, true, i == self.formation) && i != self.formation {
                self.formation = i;
                self.auto_fill(run);
            }
        }

        let formation = &run.formations[self.formation];
        view::draw_boards();
        for &pos in &formation.slots {
            let o = view::cell_origin(Side::Ally, pos);
            draw_rectangle(
                o.x + 2.0,
                o.y + 2.0,
                CELL - 4.0,
                CELL - 4.0,
                palette::OPEN_SLOT,
            );
        }

        // 預覽用的我方精靈（不含陣型加成，只用來畫位置與目標）
        let mut units: Vec<Unit> = formation
            .slots
            .iter()
            .zip(&self.slots)
            .filter_map(|(&pos, s)| s.map(|i| Unit::new(&run.party[i], Side::Ally, pos)))
            .collect();
        units.extend(self.enemies.iter().cloned());
        for u in &units {
            view::draw_unit(u);
        }
        view::draw_preview(&units);

        // 每一格的陣型加成，畫在格子最下方
        for (i, &pos) in formation.slots.iter().enumerate() {
            let label = bonus_label(&formation.slot_percents(i));
            if !label.is_empty() {
                let o = view::cell_origin(Side::Ally, pos);
                text::centered(
                    &label,
                    vec2(o.x + CELL / 2.0, o.y + CELL - 6.0),
                    12.0,
                    palette::HIGHLIGHT,
                );
            }
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
        let hint = if ready {
            format!("已擺 {placed} 隻")
        } else {
            format!("至少擺 {MIN_UNITS} 隻")
        };
        text::draw(
            &hint,
            600.0,
            515.0,
            24.0,
            if ready {
                palette::TEXT_DIM
            } else {
                palette::WARNING
            },
        );
        if ui::button(Rect::new(600.0, 525.0, 160.0, 50.0), "開戰！", ready, false) {
            return formation.place(&self.placement(run), Side::Ally).ok();
        }
        None
    }

    fn draw_party(&mut self, run: &Run) {
        text::draw(
            &format!("隊伍 {} 隻（點精靈，再點綠色格子）", run.party.len()),
            40.0,
            CARD_TOP - 10.0,
            24.0,
            palette::TEXT_DIM,
        );

        let pages = run.party.len().div_ceil(CARDS_PER_PAGE).max(1);
        self.page = self.page.min(pages - 1);
        let arrow_y = CARD_TOP + (CARD_H - 40.0) / 2.0;
        if ui::button(
            Rect::new(40.0, arrow_y, 28.0, 40.0),
            "<",
            self.page > 0,
            false,
        ) {
            self.page -= 1;
        }
        let right = CARDS_LEFT + CARDS_PER_PAGE as f32 * (CARD_W + CARD_GAP);
        if ui::button(
            Rect::new(right, arrow_y, 28.0, 40.0),
            ">",
            self.page + 1 < pages,
            false,
        ) {
            self.page += 1;
        }

        let first = self.page * CARDS_PER_PAGE;
        let on_page = first..run.party.len().min(first + CARDS_PER_PAGE);
        let mut hovered = None;
        for i in on_page {
            let sprite = &run.party[i];
            let rect = card_rect(i - first);
            if rect.contains(ui::mouse()) {
                hovered = Some(i);
            }
            let placed = self.slots.contains(&Some(i));
            let label = if placed {
                format!("{} *", sprite.name)
            } else {
                sprite.name.clone()
            };
            if ui::button(rect, &label, true, self.selected == Some(i)) {
                self.selected = if self.selected == Some(i) {
                    None
                } else {
                    Some(i)
                };
            }
        }

        let shown = self.selected.or(hovered);
        if let Some(i) = shown {
            let sprite = &run.party[i];
            let s = &sprite.stats;
            let lines = [
                format!(
                    "{}　HP {}　攻擊 {}　防禦 {}",
                    sprite.name, s.hp, s.attack, s.defense
                ),
                format!("每 {} 回合攻擊，第 {} 回合開始", s.interval, s.start_turn),
                format!(
                    "技能「{}」：每 {} 次攻擊一次",
                    sprite.skill.name, s.skill_every
                ),
            ];
            for (n, line) in lines.iter().enumerate() {
                let color = if n == 0 {
                    palette::TEXT
                } else {
                    palette::TEXT_DIM
                };
                text::draw(line, 40.0, 528.0 + n as f32 * 25.0, 24.0, color);
            }
        }
    }
}
