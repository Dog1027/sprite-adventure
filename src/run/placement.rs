//! 陣型選擇與擺位畫面：看得到敵方陣容，選陣型、把精靈放進開放的格子，預覽線顯示第一個攻擊目標。

use super::Run;
use super::ui;
use crate::battle::formation::MIN_UNITS;
use crate::battle::view::{self, CELL};
use crate::battle::{Side, Unit};
use crate::sprite::Sprite;
use macroquad::prelude::*;

const CARD_TOP: f32 = 440.0;
const CARD_W: f32 = 90.0;
const CARD_H: f32 = 60.0;

pub struct PlacementScreen {
    pub enemies: Vec<Unit>,
    formation: usize,
    /// 每一格放的是隊伍中第幾隻精靈
    slots: Vec<Option<usize>>,
    /// 選中的隊伍精靈，下一次點格子時放進去
    selected: Option<usize>,
}

impl PlacementScreen {
    pub fn new(run: &Run, enemies: Vec<Unit>) -> Self {
        let mut screen = Self {
            enemies,
            formation: 0,
            slots: Vec::new(),
            selected: None,
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
        draw_text(
            "Choose a formation and place your sprites",
            40.0,
            50.0,
            30.0,
            WHITE,
        );

        for (i, f) in run.formations.iter().enumerate() {
            let rect = Rect::new(40.0 + i as f32 * 130.0, 70.0, 120.0, 40.0);
            let label = format!("{} ({})", f.name, f.slots.len());
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
                Color::new(0.2, 0.35, 0.25, 1.0),
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
            format!("{placed} placed")
        } else {
            format!("Place at least {MIN_UNITS} ({placed} placed)")
        };
        draw_text(
            &hint,
            600.0,
            515.0,
            22.0,
            if ready { LIGHTGRAY } else { ORANGE },
        );
        if ui::button(Rect::new(600.0, 525.0, 160.0, 50.0), "Fight!", ready, false) {
            return formation.place(&self.placement(run), Side::Ally).ok();
        }
        None
    }

    fn draw_party(&mut self, run: &Run) {
        draw_text(
            "Party (click a sprite, then a green cell)",
            40.0,
            CARD_TOP - 10.0,
            22.0,
            LIGHTGRAY,
        );
        for (i, sprite) in run.party.iter().enumerate() {
            let rect = Rect::new(40.0 + i as f32 * (CARD_W + 10.0), CARD_TOP, CARD_W, CARD_H);
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

        let shown = self.selected.or_else(|| {
            run.party.iter().enumerate().position(|(i, _)| {
                let r = Rect::new(40.0 + i as f32 * (CARD_W + 10.0), CARD_TOP, CARD_W, CARD_H);
                r.contains(ui::mouse())
            })
        });
        if let Some(i) = shown {
            let s = &run.party[i].stats;
            let line1 = format!(
                "{}  HP {}  ATK {}  DEF {}",
                run.party[i].name, s.hp, s.attack, s.defense
            );
            let line2 = format!(
                "attacks every {} turn(s) from turn {}, skill every {} attacks",
                s.interval, s.start_turn, s.skill_every
            );
            draw_text(&line1, 40.0, 535.0, 22.0, WHITE);
            draw_text(&line2, 40.0, 560.0, 20.0, LIGHTGRAY);
        }
    }
}
