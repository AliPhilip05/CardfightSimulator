//! The shared geometry for drawing and clicking the tabletop.

use super::cards::{draw_card_face, draw_field_unit, draw_guard_card};
use super::drawing::{draw_centered, draw_round_panel, draw_wrapped, CREAM, MUTED, PANEL};
use crate::game::card::{Card, Player, Unit};
use crate::game::engine::MatchGame;
use macroquad::prelude::*;
use std::collections::HashMap;

// The art remains at its original proportions. Only the spaces between zones resize.
const MIN_WIDTH: f32 = 710.0;
const TABLE_HEIGHT: f32 = 720.0;
const CENTER_Y: f32 = TABLE_HEIGHT * 0.5;
const CPU_BACK_Y: f32 = 78.0;
const CPU_FRONT_Y: f32 = 221.0;
const COLUMN_FRACTIONS: [f32; 3] = [0.28, 0.50, 0.72];
const CIRCLE_RADIUS: f32 = 65.0;
const CARD_SIZE: Vec2 = Vec2::new(75.0, 108.0);
const GUARDIAN_RADII: Vec2 = Vec2::new(110.0, 45.0);
const DAMAGE_POCKET: Vec2 = Vec2::new(123.0, 271.0);
const DAMAGE_CARD_SIZE: Vec2 = Vec2::new(48.0, 70.0);
const DAMAGE_STEP_X: f32 = 57.0;
const DAMAGE_STEP_Y: f32 = 76.0;
const PAPER: Color = Color::new(0.944, 0.955, 0.958, 1.0);
const EDGE: Color = Color::new(0.606, 0.724, 0.776, 1.0);
const POCKET: Color = Color::new(0.875, 0.917, 0.936, 1.0);
const POCKET_INSET: Color = Color::new(0.842, 0.895, 0.917, 1.0);
const INK: Color = Color::new(0.094, 0.170, 0.232, 1.0);

// Art regions from the user-provided supplied-playmat.png.
const REAR_ART: Rect = Rect::new(141.0, 260.0, 132.0, 132.0);
const VANGUARD_ART: Rect = Rect::new(273.0, 92.0, 169.0, 161.0);
const GUARDIAN_ART: Rect = Rect::new(264.0, 0.0, 185.0, 95.0);
const CARD_BACK_ART: Rect = Rect::new(612.0, 144.0, 73.0, 103.0);
const DECK_ART: Rect = Rect::new(608.0, 140.0, 81.0, 110.0);
const DROP_ART: Rect = Rect::new(608.0, 269.0, 81.0, 111.0);
const TRIGGER_ART: Rect = Rect::new(15.0, 20.0, 110.0, 110.0);

#[derive(Default)]
pub(super) struct BattlefieldHighlights {
    pub(super) player_rear: [Option<Color>; 5],
    pub(super) cpu_rear: [Option<Color>; 5],
    pub(super) player_vanguard: Option<Color>,
    pub(super) cpu_vanguard: Option<Color>,
    pub(super) guardian: Option<Color>,
}

pub(super) struct BattlefieldLayout {
    bounds: Rect,
    width: f32,
    scale: f32,
}

impl BattlefieldLayout {
    pub(super) fn new(bounds: Rect) -> Self {
        let scale = (bounds.w / MIN_WIDTH)
            .min(bounds.h / TABLE_HEIGHT)
            .max(0.001);
        let height = TABLE_HEIGHT * scale;
        Self {
            bounds: Rect::new(
                bounds.x,
                bounds.y + (bounds.h - height) * 0.5,
                bounds.w,
                height,
            ),
            width: bounds.w / scale,
            scale,
        }
    }

    fn point(&self, local: Vec2) -> Vec2 {
        vec2(self.bounds.x, self.bounds.y) + local * self.scale
    }

    fn rect(&self, center: Vec2, size: Vec2) -> Rect {
        let top_left = self.point(center - size * 0.5);
        Rect::new(
            top_left.x,
            top_left.y,
            size.x * self.scale,
            size.y * self.scale,
        )
    }

    fn side_point(&self, player_point: Vec2, cpu: bool) -> Vec2 {
        if cpu {
            vec2(self.width, TABLE_HEIGHT) - player_point
        } else {
            player_point
        }
    }

    fn row_center(&self, column: usize, front: bool, cpu: bool) -> Vec2 {
        let cpu_y = if front { CPU_FRONT_Y } else { CPU_BACK_Y };
        let player_x = self.width * COLUMN_FRACTIONS[column];
        self.side_point(vec2(player_x, TABLE_HEIGHT - cpu_y), cpu)
    }

    fn rear_center(&self, slot: usize, cpu: bool) -> Vec2 {
        const SLOT_POSITION: [(usize, bool); 5] =
            [(0, false), (0, true), (1, false), (2, true), (2, false)];
        let (column, front) = SLOT_POSITION[slot];
        self.row_center(column, front, cpu)
    }

    fn vanguard_center(&self, cpu: bool) -> Vec2 {
        self.row_center(1, true, cpu)
    }

    fn guardian_center(&self) -> Vec2 {
        vec2(self.width * 0.5, CENTER_Y)
    }

    fn damage_center(&self, cpu: bool) -> Vec2 {
        self.side_point(vec2(self.width * 0.10, 576.0), cpu)
    }

    fn damage_card(&self, index: usize, cpu: bool) -> Rect {
        let column = (index % 2) as f32 - 0.5;
        let row = (index / 2) as f32 - 1.0;
        let directions = if cpu { -1.0 } else { 1.0 };
        let offset = vec2(column * DAMAGE_STEP_X, row * DAMAGE_STEP_Y + 10.0) * directions;
        self.rect(self.damage_center(cpu) + offset, DAMAGE_CARD_SIZE)
    }

    fn unit_at(&self, mouse: Vec2, center: Vec2) -> bool {
        self.point(center).distance(mouse) <= CIRCLE_RADIUS * self.scale
            || self.rect(center, CARD_SIZE).contains(mouse)
    }

    pub(super) fn rear_at(&self, mouse: Vec2, cpu: bool) -> Option<usize> {
        (0..5).find(|&slot| self.unit_at(mouse, self.rear_center(slot, cpu)))
    }

    pub(super) fn vanguard_at(&self, mouse: Vec2, cpu: bool) -> bool {
        self.unit_at(mouse, self.vanguard_center(cpu))
    }

    pub(super) fn guardian_at(&self, mouse: Vec2) -> bool {
        let delta = mouse - self.point(self.guardian_center());
        (delta / (GUARDIAN_RADII * self.scale)).length_squared() <= 1.0
    }

    fn label(&self, text: &str, local: Vec2, size: f32, color: Color) {
        let point = self.point(local);
        draw_centered(
            text,
            point.x,
            point.y,
            0.0,
            (size * self.scale).max(8.0),
            color,
        );
    }
}

pub(super) fn draw_battlefield(
    game: &MatchGame,
    card_art: &HashMap<String, Texture2D>,
    layout: &BattlefieldLayout,
    playmat: &Texture2D,
    highlights: &BattlefieldHighlights,
    guardians: &[Card],
) {
    draw_table_surface(layout);
    let mouse = mouse_position().into();
    let mut damage_preview = None;
    for (cpu, side, rear_highlights, vanguard_highlight) in [
        (
            true,
            &game.cpu,
            &highlights.cpu_rear,
            highlights.cpu_vanguard,
        ),
        (
            false,
            &game.player,
            &highlights.player_rear,
            highlights.player_vanguard,
        ),
    ] {
        draw_side_zones(side, layout, cpu, playmat);
        if let Some((card, area)) = draw_damage(side, layout, cpu, playmat, card_art, mouse) {
            damage_preview = Some((card, area, cpu));
        }
        for (slot, highlight) in rear_highlights.iter().enumerate() {
            let center = layout.rear_center(slot, cpu);
            draw_mat_art(layout, center, REAR_ART, 1.0, cpu, playmat);
            draw_circle_unit(
                layout,
                center,
                side.rear[slot].as_ref(),
                *highlight,
                card_art,
            );
        }
        let center = layout.vanguard_center(cpu);
        draw_mat_art(layout, center, VANGUARD_ART, 1.0, cpu, playmat);
        draw_circle_unit(
            layout,
            center,
            side.vanguard.as_ref(),
            vanguard_highlight,
            card_art,
        );
        if !side.soul.is_empty() {
            let direction = if cpu { -1.0 } else { 1.0 };
            draw_count(
                layout,
                center + vec2(93.0, direction * 33.0),
                &format!("SOUL {}", side.soul.len()),
            );
        }
    }
    draw_guardians(layout, playmat, highlights.guardian, guardians, card_art);
    if let Some((card, area, cpu)) = damage_preview {
        draw_damage_preview(layout, area, cpu, card, card_art);
    }
}

fn draw_table_surface(layout: &BattlefieldLayout) {
    let Rect { x, y, w, h } = layout.bounds;
    let shadow = Color::new(0.015, 0.026, 0.043, 0.62);
    draw_rectangle(x - 4.0, y - 3.0, w + 8.0, h + 9.0, shadow);
    draw_rectangle(x, y, w, h, PAPER);
    draw_rectangle_lines(x, y, w, h, 1.0, EDGE);

    let inset = 7.0 * layout.scale;
    draw_rectangle_lines(
        x + inset,
        y + inset,
        w - inset * 2.0,
        h - inset * 2.0,
        1.0,
        tint(EDGE, 0.52),
    );
    let center = layout.point(layout.guardian_center());
    let opening = 114.0 * layout.scale;
    draw_line(
        x + inset,
        center.y,
        center.x - opening,
        center.y,
        1.0,
        tint(EDGE, 0.70),
    );
    draw_line(
        center.x + opening,
        center.y,
        x + w - inset,
        center.y,
        1.0,
        tint(EDGE, 0.70),
    );
}

fn draw_mat_art(
    layout: &BattlefieldLayout,
    center: Vec2,
    source: Rect,
    factor: f32,
    cpu: bool,
    playmat: &Texture2D,
) {
    let dest = layout.rect(center, source.size() * factor);
    draw_texture_ex(
        playmat,
        dest.x,
        dest.y,
        PAPER,
        DrawTextureParams {
            dest_size: Some(dest.size()),
            source: Some(source),
            rotation: if cpu { std::f32::consts::PI } else { 0.0 },
            ..Default::default()
        },
    );
}

fn draw_circle_unit(
    layout: &BattlefieldLayout,
    center: Vec2,
    unit: Option<&Unit>,
    highlight: Option<Color>,
    card_art: &HashMap<String, Texture2D>,
) {
    if let Some(color) = highlight {
        let point = layout.point(center);
        let radius = CIRCLE_RADIUS * layout.scale;
        let hovered = layout.unit_at(mouse_position().into(), center);
        draw_circle(
            point.x,
            point.y,
            radius,
            tint(color, if hovered { 0.32 } else { 0.12 }),
        );
        draw_circle_lines(point.x, point.y, radius + 1.0, 3.5, tint(INK, 0.89));
        draw_circle_lines(point.x, point.y, radius + 1.0, 1.7, color);
    }
    if let Some(unit) = unit {
        draw_field_unit(layout.rect(center, CARD_SIZE), unit, card_art);
    }
}

fn draw_guardians(
    layout: &BattlefieldLayout,
    playmat: &Texture2D,
    highlight: Option<Color>,
    guardians: &[Card],
    card_art: &HashMap<String, Texture2D>,
) {
    let center = layout.guardian_center();
    let point = layout.point(center);
    let radii = GUARDIAN_RADII * layout.scale;
    draw_mat_art(
        layout,
        center + vec2(0.0, 7.5),
        GUARDIAN_ART,
        0.90,
        false,
        playmat,
    );
    draw_ellipse_lines(
        point.x,
        point.y,
        radii.x,
        radii.y,
        0.0,
        1.0,
        tint(EDGE, 0.75),
    );
    layout.label("GUARDIAN CIRCLE", center + vec2(0.0, 64.0), 12.0, INK);
    if let Some(color) = highlight {
        let hovered = layout.guardian_at(mouse_position().into());
        draw_ellipse(
            point.x,
            point.y,
            radii.x,
            radii.y,
            0.0,
            tint(color, if hovered { 0.32 } else { 0.14 }),
        );
        draw_ellipse_lines(point.x, point.y, radii.x, radii.y, 0.0, 3.3, INK);
        draw_ellipse_lines(point.x, point.y, radii.x, radii.y, 0.0, 1.7, color);
    }
    let size = vec2(82.0, 57.0);
    let intervals = guardians.len().saturating_sub(1) as f32;
    let spacing = (118.0 / intervals.max(1.0)).min(size.x + 4.0);
    let start = center - vec2(intervals * spacing * 0.5, 0.0);
    for (index, card) in guardians.iter().enumerate() {
        let center = start + vec2(index as f32 * spacing, 0.0);
        draw_guard_card(layout.rect(center, size), card, card_art);
    }
}

fn draw_damage<'a>(
    side: &'a Player,
    layout: &BattlefieldLayout,
    cpu: bool,
    playmat: &Texture2D,
    card_art: &HashMap<String, Texture2D>,
    mouse: Vec2,
) -> Option<(&'a Card, Rect)> {
    let center = layout.damage_center(cpu);
    let panel = layout.rect(center, DAMAGE_POCKET);
    draw_rectangle(panel.x, panel.y, panel.w, panel.h, POCKET);
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 1.0, EDGE);
    // Keep the labels upright. Numbering still reflects the fighter's own side.
    let label_y = if cpu { 124.0 } else { -108.0 };
    layout.label(
        &format!("DAMAGE  {} / 6", side.damage.len()),
        center + vec2(0.0, label_y),
        14.0,
        INK,
    );
    let mut hovered = None;
    for index in 0..6 {
        let bounds = layout.damage_card(index, cpu);
        if let Some(damage) = side.damage.get(index) {
            if damage.face_up {
                draw_card_face(bounds, &damage.card, false, card_art);
                if bounds.contains(mouse) {
                    draw_rectangle_lines(
                        bounds.x - 1.0,
                        bounds.y - 1.0,
                        bounds.w + 2.0,
                        bounds.h + 2.0,
                        2.0,
                        INK,
                    );
                    hovered = Some((&damage.card, bounds));
                }
            } else {
                draw_texture_ex(
                    playmat,
                    bounds.x,
                    bounds.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(bounds.size()),
                        source: Some(CARD_BACK_ART),
                        ..Default::default()
                    },
                );
            }
        } else {
            draw_rectangle(bounds.x, bounds.y, bounds.w, bounds.h, POCKET_INSET);
            draw_rectangle_lines(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                1.0,
                tint(EDGE, 0.72),
            );
            draw_centered(
                &(index + 1).to_string(),
                bounds.x,
                bounds.center().y + 4.0,
                bounds.w,
                12.0 * layout.scale,
                tint(INK, 0.53),
            );
        }
    }
    hovered
}

fn draw_damage_preview(
    layout: &BattlefieldLayout,
    area: Rect,
    cpu: bool,
    card: &Card,
    card_art: &HashMap<String, Texture2D>,
) {
    let factor = layout.scale.clamp(0.56, 1.18);
    let width = 145.0 * factor;
    let height = 209.0 * factor;
    let pad = 8.0;
    let caption = 25.0;
    let min_x = layout.bounds.left() + pad;
    let max_x = (layout.bounds.right() - width - pad * 2.0).max(min_x);
    let x = if cpu {
        area.left() - width - pad * 2.0
    } else {
        area.right() + pad * 2.0
    }
    .clamp(min_x, max_x);
    let top = layout.bounds.top() + pad;
    let y = (area.center().y - height * 0.5).clamp(
        top,
        (layout.bounds.bottom() - height - caption - pad * 2.0).max(top),
    );
    draw_round_panel(
        x - pad,
        y - pad,
        width + pad * 2.0,
        height + caption + pad,
        PANEL,
        EDGE,
    );
    draw_card_face(Rect::new(x, y, width, height), card, false, card_art);
    draw_wrapped(card.name, x, y + height + 11.0, width, 10.5, CREAM, 2);
}

fn draw_side_zones(side: &Player, layout: &BattlefieldLayout, cpu: bool, playmat: &Texture2D) {
    let player_x = layout.width * 0.90;
    let deck = layout.side_point(vec2(player_x, 634.0), cpu);
    let drop = layout.side_point(vec2(player_x, 494.0), cpu);
    let trigger = layout.side_point(vec2(layout.width * 0.10, 387.0), cpu);
    draw_mat_art(layout, deck, DECK_ART, 0.84, cpu, playmat);
    draw_mat_art(layout, drop, DROP_ART, 0.84, cpu, playmat);
    draw_mat_art(layout, trigger, TRIGGER_ART, 0.72, cpu, playmat);
    draw_count(
        layout,
        deck + vec2(0.0, 58.0),
        &format!("DECK  {}", side.deck.len()),
    );
    draw_count(
        layout,
        drop + vec2(0.0, 58.0),
        &format!("DROP  {}", side.drop_zone.len()),
    );
    layout.label("TRIGGER", trigger + vec2(0.0, 52.0), 11.0, INK);
    if let Some(check) = side.last_check.as_deref() {
        let bounds = layout.rect(trigger, vec2(73.0, 62.0));
        draw_rectangle(bounds.x, bounds.y, bounds.w, bounds.h, tint(INK, 0.90));
        draw_wrapped(
            check,
            bounds.x + 3.0,
            bounds.y + 13.0 * layout.scale,
            (bounds.w - 6.0).max(1.0),
            (10.0 * layout.scale).max(8.0),
            CREAM,
            4,
        );
    }
}

fn draw_count(layout: &BattlefieldLayout, center: Vec2, text: &str) {
    let size = (11.0 * layout.scale).max(8.0);
    let point = layout.point(center);
    let width = measure_text(text, None, size as u16, 1.0).width + 13.0;
    let height = size + 5.0;
    draw_rectangle(
        point.x - width * 0.5,
        point.y - height * 0.5,
        width,
        height,
        INK,
    );
    draw_centered(
        text,
        point.x - width * 0.5,
        point.y + size * 0.32,
        width,
        size,
        WHITE,
    );
}

fn tint(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_front_circle_aligns_with_its_back_circle() {
        let layout = BattlefieldLayout::new(Rect::new(16.0, 74.0, 1408.0, 681.0));
        for cpu in [true, false] {
            let front = [
                layout.rear_center(1, cpu),
                layout.vanguard_center(cpu),
                layout.rear_center(3, cpu),
            ];
            let back = [0, 2, 4].map(|slot| layout.rear_center(slot, cpu));
            for (front, back) in front.into_iter().zip(back) {
                assert_eq!(front.x, back.x);
                let expected = if cpu {
                    CPU_FRONT_Y
                } else {
                    TABLE_HEIGHT - CPU_FRONT_Y
                };
                assert_eq!(front.y, expected);
                let expected = if cpu {
                    CPU_BACK_Y
                } else {
                    TABLE_HEIGHT - CPU_BACK_Y
                };
                assert_eq!(back.y, expected);
            }
        }
        assert_eq!(
            layout.vanguard_center(true).x,
            layout.vanguard_center(false).x
        );
    }

    #[test]
    fn shrinking_the_table_preserves_unit_and_guardian_hit_areas() {
        let layout = BattlefieldLayout::new(Rect::new(16.0, 74.0, 744.0, 440.0));
        for cpu in [true, false] {
            for slot in 0..5 {
                let center = layout.point(layout.rear_center(slot, cpu));
                assert_eq!(layout.rear_at(center, cpu), Some(slot));
                assert_eq!(layout.rear_at(center, !cpu), None);
                assert!(!layout.vanguard_at(center, cpu));
            }
            assert!(layout.vanguard_at(layout.point(layout.vanguard_center(cpu)), cpu));
        }
        let center = layout.guardian_center();
        assert!(layout.guardian_at(layout.point(center)));
        assert!(!layout.guardian_at(layout.point(center + GUARDIAN_RADII)));
    }

    #[test]
    fn all_six_damage_cards_fit_and_remain_fully_visible() {
        let layout = BattlefieldLayout::new(Rect::new(0.0, 0.0, MIN_WIDTH, TABLE_HEIGHT));
        for cpu in [true, false] {
            let pocket = layout.rect(layout.damage_center(cpu), DAMAGE_POCKET);
            for index in 0..6 {
                let card = layout.damage_card(index, cpu);
                assert!(pocket.contains(vec2(card.left(), card.top())));
                assert!(pocket.contains(vec2(card.right(), card.bottom())));
                for other in index + 1..6 {
                    assert!(!card.overlaps(&layout.damage_card(other, cpu)));
                }
            }
        }
    }
}
