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
/// 按下後移動超過這個距離才算拖曳，否則放開時當成點一下
const DRAG_THRESHOLD: f32 = 8.0;
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

/// 正在拖曳（或剛按下）的精靈
struct Drag {
    /// 隊伍中第幾隻
    member: usize,
    /// 從陣型的第幾格拿起來；`None` 表示從下方的隊伍卡片
    from: Option<usize>,
    start: Vec2,
    /// 是否已經移動到算是拖曳
    moved: bool,
}

/// 把精靈放到第 `target` 格（`None` 表示放到棋盤外）。
/// - 目標格有別的精靈：從格子拖來的就交換位置，從隊伍拖來的就把原本的換下來
/// - 從格子拖到棋盤外：拿下來
fn drop_on(slots: &mut [Option<usize>], member: usize, from: Option<usize>, target: Option<usize>) {
    let Some(target) = target else {
        if from.is_some() {
            for s in slots.iter_mut().filter(|s| **s == Some(member)) {
                *s = None;
            }
        }
        return;
    };
    let displaced = slots[target].filter(|&d| d != member);
    for s in slots.iter_mut().filter(|s| **s == Some(member)) {
        *s = None;
    }
    slots[target] = Some(member);
    if let (Some(from), Some(other)) = (from, displaced) {
        slots[from] = Some(other);
    }
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
    drag: Option<Drag>,
}

impl PlacementScreen {
    pub fn new(run: &Run, enemies: Vec<Unit>) -> Self {
        let mut screen = Self {
            enemies,
            formation: 0,
            slots: Vec::new(),
            selected: None,
            page: 0,
            drag: None,
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

        self.handle_input(run);
        let dragging = self.drag.as_ref().filter(|d| d.moved);

        let formation = &run.formations[self.formation];
        view::draw_boards();
        // 拖曳中的目標格
        let target = dragging.and_then(|_| self.slot_at(run, ui::mouse()));
        // 開放的格子與加成（空格時加成寫在地上）
        for (i, &pos) in formation.slots.iter().enumerate() {
            let (fill, border) = if target == Some(i) {
                (
                    gfx::with_alpha(palette::OPEN_SLOT_BORDER, 0.45),
                    palette::SUCCESS,
                )
            } else {
                (palette::OPEN_SLOT, palette::OPEN_SLOT_BORDER)
            };
            view::draw_slot(Side::Ally, pos, fill, border);
            let label = bonus_label(&formation.slot_percents(i));
            if self.slots[i].is_none() && !label.is_empty() {
                let c = view::cell_center(Side::Ally, pos);
                text::centered(&label, c, 18.0, palette::HIGHLIGHT);
            }
        }

        // 我方精靈（含陣型加成，預覽才準）；從格子拖起來的那隻先不畫在棋盤上
        let mut shown = self.slots.clone();
        if let Some(Drag { from: Some(f), .. }) = dragging {
            shown[*f] = None;
        }
        let placement: Vec<Option<&Sprite>> =
            shown.iter().map(|s| s.map(|i| &run.party[i])).collect();
        let mut units = formation.place(&placement, Side::Ally).unwrap_or_default();
        units.extend(self.enemies.iter().cloned());
        // 由後往前畫；我方的格子有加成時，左下角顯示加成（取代血量數字）
        for i in view::draw_order(&units) {
            let u = &units[i];
            let bonus = formation
                .slots
                .iter()
                .position(|&p| p == u.pos && u.side == Side::Ally)
                .map(|i| bonus_label(&formation.slot_percents(i)))
                .filter(|l| !l.is_empty());
            view::draw_unit(u, bonus.as_deref());
        }
        view::draw_preview(&units);
        if let Some(i) = self.selected
            && let Some(slot) = self.slots.iter().position(|&s| s == Some(i))
        {
            let r = view::unit_rect(Side::Ally, formation.slots[slot]);
            gfx::rounded_rect_lines(r, 8.0, 3.0, palette::BUTTON_ACTIVE);
        }

        self.draw_party(run);
        // 拖曳中的精靈跟著手指，畫在手指上方一點，才不會被手指擋住
        if let Some(d) = self.drag.as_ref().filter(|d| d.moved) {
            let p = ui::mouse() - vec2(0.0, 36.0);
            let r = Rect::new(p.x - CARD_W / 2.0, p.y - CARD_H / 2.0, CARD_W, CARD_H);
            gfx::shadow(r, 10.0);
            gfx::rounded_rect(r, 10.0, gfx::with_alpha(palette::BUTTON_ACTIVE, 0.9));
            text::centered(&run.party[d.member].name, r.center(), 24.0, palette::TEXT);
        }

        let placed = self.slots.iter().flatten().count();
        let ready = placed >= MIN_UNITS;
        let label = if ready {
            format!("開戰（{placed} 隻上場）")
        } else {
            format!("至少擺 {MIN_UNITS} 隻")
        };
        if ui::button(BOTTOM_BUTTON, &label, Style::Primary, ready) {
            let formation = &run.formations[self.formation];
            return formation.place(&self.placement(run), Side::Ally).ok();
        }
        None
    }

    /// 畫面座標落在陣型的第幾格
    fn slot_at(&self, run: &Run, p: Vec2) -> Option<usize> {
        let pos = view::cell_at(Side::Ally, p)?;
        run.formations[self.formation]
            .slots
            .iter()
            .position(|&s| s == pos)
    }

    /// 這一頁的隊伍卡片中，畫面座標落在哪一隻
    fn card_at(&self, run: &Run, p: Vec2) -> Option<usize> {
        let first = self.page * CARDS_PER_PAGE;
        (first..run.party.len().min(first + CARDS_PER_PAGE))
            .find(|&i| card_rect(i - first).contains(p))
    }

    /// 按下、拖曳、放開。沒有拖動就放開時當成點一下（點選的操作照舊）
    fn handle_input(&mut self, run: &Run) {
        let p = ui::mouse();
        if ui::clicked() {
            if let Some(member) = self.card_at(run, p) {
                self.drag = Some(Drag {
                    member,
                    from: None,
                    start: p,
                    moved: false,
                });
            } else if let Some(slot) = self.slot_at(run, p) {
                match self.slots[slot] {
                    // 有選中的精靈時，點格子就放進去
                    _ if self.selected.is_some() => {
                        let member = self.selected.take().unwrap();
                        drop_on(&mut self.slots, member, None, Some(slot));
                    }
                    Some(member) => {
                        self.drag = Some(Drag {
                            member,
                            from: Some(slot),
                            start: p,
                            moved: false,
                        })
                    }
                    None => {}
                }
            }
        }

        if let Some(d) = &mut self.drag
            && ui::held()
            && p.distance(d.start) > DRAG_THRESHOLD
        {
            d.moved = true;
            self.selected = None;
        }

        if ui::released()
            && let Some(d) = self.drag.take()
        {
            if d.moved {
                let target = self.slot_at(run, p);
                drop_on(&mut self.slots, d.member, d.from, target);
            } else if d.from.is_some() {
                // 點格子裡的精靈：拿起來，接著點別的格子放下
                drop_on(&mut self.slots, d.member, d.from, None);
                self.selected = Some(d.member);
            } else {
                // 點隊伍卡片：選取／取消選取
                self.selected = if self.selected == Some(d.member) {
                    None
                } else {
                    Some(d.member)
                };
            }
        }
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
            // 點擊與拖曳在 handle_input 處理
            ui::draw_button(rect, &run.party[i].name, style, true);
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
                text::draw(&line2, MARGIN, info_y + 22.0, 18.0, palette::TEXT_DIM);
            }
            None => {
                text::draw(
                    "把精靈拖到綠色格子，或點精靈再點格子",
                    MARGIN,
                    info_y,
                    18.0,
                    palette::TEXT_DIM,
                );
                text::draw(
                    "拖格子裡的精靈可以換位置，拖出去就拿下",
                    MARGIN,
                    info_y + 22.0,
                    18.0,
                    palette::TEXT_MUTED,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::drop_on;

    #[test]
    fn drop_from_party_replaces_occupant() {
        let mut slots = vec![Some(0), None, Some(1)];
        drop_on(&mut slots, 2, None, Some(0));
        assert_eq!(slots, vec![Some(2), None, Some(1)]);
    }

    #[test]
    fn drop_from_party_moves_member_already_placed() {
        let mut slots = vec![Some(0), None, Some(1)];
        drop_on(&mut slots, 1, None, Some(1));
        assert_eq!(slots, vec![Some(0), Some(1), None]);
    }

    #[test]
    fn drop_between_slots_swaps() {
        let mut slots = vec![Some(0), None, Some(1)];
        drop_on(&mut slots, 0, Some(0), Some(2));
        assert_eq!(slots, vec![Some(1), None, Some(0)]);
    }

    #[test]
    fn drop_onto_empty_slot_moves() {
        let mut slots = vec![Some(0), None, Some(1)];
        drop_on(&mut slots, 0, Some(0), Some(1));
        assert_eq!(slots, vec![None, Some(0), Some(1)]);
    }

    #[test]
    fn drop_outside_board_removes() {
        let mut slots = vec![Some(0), None, Some(1)];
        drop_on(&mut slots, 1, Some(2), None);
        assert_eq!(slots, vec![Some(0), None, None]);
        // 從隊伍拖到外面：什麼都不變
        drop_on(&mut slots, 0, None, None);
        assert_eq!(slots, vec![Some(0), None, None]);
    }

    #[test]
    fn drop_back_on_same_slot_keeps_it() {
        let mut slots = vec![Some(0), None, Some(1)];
        drop_on(&mut slots, 0, Some(0), Some(0));
        assert_eq!(slots, vec![Some(0), None, Some(1)]);
    }
}
