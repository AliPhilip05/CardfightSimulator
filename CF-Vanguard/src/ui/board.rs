//! The shared geometry for drawing and clicking the tabletop.

use super::cards::{draw_card_face, draw_field_unit, draw_guard_card};
use super::drawing::{draw_centered, draw_round_panel, draw_wrapped, CREAM, PANEL};
use crate::game::card::{Card, Player, Unit};
use crate::game::engine::MatchGame;
use macroquad::prelude::*;
use std::collections::HashMap;

// The art remains at its original proportions. Only the spaces between zones resize.
const MIN_WIDTH: f32 = 1100.0;
const TABLE_HEIGHT: f32 = 720.0;
const CENTER_Y: f32 = TABLE_HEIGHT * 0.5;
const CPU_BACK_Y: f32 = 78.0;
const CPU_FRONT_Y: f32 = 221.0;
const COLUMN_FRACTIONS: [f32; 3] = [0.32, 0.50, 0.68];
const CIRCLE_RADIUS: f32 = 65.0;
const CARD_SIZE: Vec2 = Vec2::new(75.0, 108.0);
const GUARDIAN_RADII: Vec2 = Vec2::new(110.0, 50.0);
const DAMAGE_POCKET: Vec2 = Vec2::new(132.0, 250.0);
const DAMAGE_CARD_SIZE: Vec2 = Vec2::new(CARD_SIZE.y, CARD_SIZE.x);
const DAMAGE_STEP: f32 = 27.0;
const EDGE: Color = Color::from_rgba(77, 148, 193, 255);
const POCKET: Color = Color::from_rgba(14, 23, 34, 255);
const INK: Color = Color::from_rgba(212, 231, 243, 255);

// Transparent circle art from the downloaded community template. Auxiliary
// zones follow Bushiroad's classic guide, rather than its overDress additions.
const REAR_ART: Rect = Rect::new(370.0, 275.0, 282.0, 282.0);
const VANGUARD_ART: Rect = Rect::new(690.0, 245.0, 350.0, 350.0);

pub(super) struct TabletopArt {
    pub zones: Texture2D,
    pub aichi_sleeve: Texture2D,
    pub kai_sleeve: Texture2D,
}

impl TabletopArt {
    fn sleeve(&self, cpu: bool) -> &Texture2D {
        if cpu {
            &self.kai_sleeve
        } else {
            &self.aichi_sleeve
        }
    }
}

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
        self.side_point(vec2(self.width * 0.10, 580.0), cpu)
    }

    fn damage_card(&self, index: usize, cpu: bool) -> Rect {
        let directions = if cpu { -1.0 } else { 1.0 };
        let offset = vec2(0.0, (index as f32 - 2.5) * DAMAGE_STEP) * directions;
        self.rect(self.damage_center(cpu) + offset, DAMAGE_CARD_SIZE)
    }

    pub(super) fn damage_at(&self, mouse: Vec2, count: usize, cpu: bool) -> Option<usize> {
        // Later cards are painted on top; the exposed strip still selects each
        // earlier card. Keep this order identical to the rendering order.
        (0..count.min(6))
            .rev()
            .find(|&index| self.damage_card(index, cpu).contains(mouse))
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
    art: &TabletopArt,
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
        draw_side_zones(side, layout, cpu, art, card_art);
        if let Some((card, area)) = draw_damage(
            side,
            layout,
            cpu,
            art,
            card_art,
            mouse,
            game.can_flip_damage(),
        ) {
            damage_preview = Some((card, area, cpu));
        }
        for (slot, highlight) in rear_highlights.iter().enumerate() {
            let center = layout.rear_center(slot, cpu);
            draw_mat_art(layout, center, REAR_ART, 0.47, cpu, &art.zones);
            layout.label(
                "R",
                center + vec2(0.0, 8.0),
                28.0,
                Color::from_rgba(246, 209, 75, 255),
            );
            draw_circle_unit(
                layout,
                center,
                side.rear[slot].as_ref(),
                *highlight,
                card_art,
            );
        }
        let center = layout.vanguard_center(cpu);
        draw_mat_art(layout, center, VANGUARD_ART, 0.48, cpu, &art.zones);
        layout.label(
            "V",
            center + vec2(0.0, 10.0),
            34.0,
            Color::from_rgba(98, 170, 255, 255),
        );
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
    draw_guardians(layout, &art.zones, highlights.guardian, guardians, card_art);
    if let Some((card, area, cpu)) = damage_preview {
        draw_damage_preview(layout, area, cpu, card, card_art);
    }
}

fn draw_table_surface(layout: &BattlefieldLayout) {
    let Rect { x, y, w, h } = layout.bounds;
    draw_rectangle(
        x - 5.0,
        y - 5.0,
        w + 10.0,
        h + 10.0,
        Color::from_rgba(5, 17, 32, 255),
    );
    draw_rectangle(x, y, w, h, Color::from_rgba(5, 8, 13, 255));
    // Ride to Victory's black arena, blue edge lighting and metallic rails.
    for row in 0..24 {
        let shade = 11 + ((row as f32 / 24.0 * std::f32::consts::PI).sin() * 6.0) as u8;
        draw_rectangle(
            x + 6.0,
            y + row as f32 * h / 24.0,
            w - 12.0,
            h / 24.0 + 1.0,
            Color::from_rgba(shade, shade + 3, shade + 7, 255),
        );
    }
    for column in 0..=20 {
        let px = x + column as f32 * w / 20.0;
        draw_line(
            px,
            y + 8.0,
            px,
            y + h - 8.0,
            1.0,
            Color::from_rgba(36, 67, 92, 28),
        );
    }
    for row in 0..=12 {
        let py = y + row as f32 * h / 12.0;
        draw_line(
            x + 8.0,
            py,
            x + w - 8.0,
            py,
            1.0,
            Color::from_rgba(36, 67, 92, 28),
        );
    }
    draw_rectangle_lines(x, y, w, h, 3.0, Color::from_rgba(99, 127, 150, 255));
    draw_rectangle_lines(
        x + 5.0,
        y + 5.0,
        w - 10.0,
        h - 10.0,
        1.5,
        Color::from_rgba(52, 176, 233, 220),
    );
    for cpu in [false, true] {
        for column in 0..3 {
            let back = layout.point(layout.row_center(column, false, cpu));
            let front = layout.point(layout.row_center(column, true, cpu));
            draw_line(
                back.x,
                back.y,
                front.x,
                front.y,
                2.0 * layout.scale,
                Color::from_rgba(83, 106, 122, 95),
            );
        }
    }
    let center = layout.point(layout.guardian_center());
    let opening = 122.0 * layout.scale;
    for direction in [-1.0, 1.0] {
        let start = center + vec2(direction * opening, 0.0);
        let end_x = if direction < 0.0 {
            x + 12.0
        } else {
            x + w - 12.0
        };
        draw_line(
            start.x,
            center.y,
            end_x,
            center.y,
            2.0,
            Color::from_rgba(100, 145, 175, 155),
        );
        for offset in [-6.0, 6.0] {
            draw_line(
                start.x + direction * 8.0,
                center.y + offset,
                end_x,
                center.y + offset,
                1.0,
                Color::from_rgba(34, 122, 179, 100),
            );
        }
    }
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
    let color = if source == REAR_ART {
        Color::from_rgba(230, 172, 30, 255)
    } else if source == VANGUARD_ART {
        Color::from_rgba(42, 112, 235, 255)
    } else {
        Color::from_rgba(47, 153, 147, 255)
    };
    // The source is a low-opacity overlay intended for custom playmats.
    // Layer it to keep the printed circles legible on the dark arena.
    for _ in 0..4 {
        draw_texture_ex(
            playmat,
            dest.x,
            dest.y,
            color,
            DrawTextureParams {
                dest_size: Some(dest.size()),
                source: Some(source),
                rotation: if cpu { std::f32::consts::PI } else { 0.0 },
                ..Default::default()
            },
        );
    }
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
    // Use a complete ring, stretched to the Guardian Circle's oval. The old
    // source rectangle contained only the lower arc of the printed template.
    let dest = layout.rect(center, GUARDIAN_RADII * 2.0);
    for _ in 0..4 {
        draw_texture_ex(
            playmat,
            dest.x,
            dest.y,
            Color::from_rgba(76, 227, 125, 255),
            DrawTextureParams {
                dest_size: Some(dest.size()),
                source: Some(REAR_ART),
                ..Default::default()
            },
        );
    }
    draw_ellipse_lines(
        point.x,
        point.y,
        radii.x,
        radii.y,
        0.0,
        1.2,
        Color::from_rgba(81, 220, 134, 180),
    );
    layout.label(
        "G",
        center + vec2(0.0, 9.0),
        29.0,
        Color::from_rgba(116, 244, 151, 255),
    );
    layout.label("GUARDIAN CIRCLE", center + vec2(0.0, 65.0), 11.0, INK);
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
    art: &TabletopArt,
    card_art: &HashMap<String, Texture2D>,
    mouse: Vec2,
    can_flip: bool,
) -> Option<(&'a Card, Rect)> {
    let center = layout.damage_center(cpu);
    let panel = layout.rect(center, DAMAGE_POCKET);
    draw_rectangle(panel.x, panel.y, panel.w, panel.h, POCKET);
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 1.0, EDGE);
    let label_y = if cpu { 122.0 } else { -115.0 };
    layout.label(
        &format!("DAMAGE {}/6", side.damage.len()),
        center + vec2(0.0, label_y),
        12.0,
        INK,
    );
    // One printed pocket with an overlapping stack, not six independent slots.
    if side.damage.is_empty() {
        layout.label("DAMAGE", center, 11.0, tint(INK, 0.45));
        layout.label("ZONE", center + vec2(0.0, 16.0), 11.0, tint(INK, 0.45));
    }
    let hovered_index = layout.damage_at(mouse, side.damage.len(), cpu);
    let mut hovered = None;
    for (index, damage) in side.damage.iter().take(6).enumerate() {
        let bounds = layout.damage_card(index, cpu);
        if damage.face_up {
            draw_card_face(bounds, &damage.card, true, card_art);
        } else {
            let size = vec2(bounds.h, bounds.w);
            let origin = bounds.center() - size * 0.5;
            draw_texture_ex(
                art.sleeve(cpu),
                origin.x,
                origin.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(size),
                    rotation: std::f32::consts::FRAC_PI_2
                        + if cpu { std::f32::consts::PI } else { 0.0 },
                    ..Default::default()
                },
            );
        }
        if hovered_index == Some(index) {
            if damage.face_up {
                hovered = Some((&damage.card, bounds));
            }
            if !cpu && can_flip {
                draw_rectangle_lines(
                    bounds.x - 2.0,
                    bounds.y - 2.0,
                    bounds.w + 4.0,
                    bounds.h + 4.0,
                    2.0,
                    Color::from_rgba(96, 214, 255, 255),
                );
            }
        }
    }
    let resources = center + vec2(0.0, if cpu { -116.0 } else { 116.0 });
    draw_count(
        layout,
        resources,
        &format!("CB {} / {}", side.face_up_damage(), side.damage.len()),
    );
    if !cpu && can_flip {
        if let Some(index) = hovered_index {
            draw_count(
                layout,
                center + vec2(0.0, -149.0),
                if side.damage[index].face_up {
                    "CLICK: COUNTER-BLAST"
                } else {
                    "CLICK: TURN FACE UP"
                },
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

fn draw_sleeve(bounds: Rect, sleeve: &Texture2D, cpu: bool) {
    draw_texture_ex(
        sleeve,
        bounds.x,
        bounds.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(bounds.size()),
            rotation: if cpu { std::f32::consts::PI } else { 0.0 },
            ..Default::default()
        },
    );
}

fn draw_side_zones(
    side: &Player,
    layout: &BattlefieldLayout,
    cpu: bool,
    art: &TabletopArt,
    card_art: &HashMap<String, Texture2D>,
) {
    let player_x = layout.width * 0.90;
    let deck = layout.side_point(vec2(player_x, 511.0), cpu);
    let drop = layout.side_point(vec2(player_x, 641.0), cpu);
    let trigger = layout.side_point(vec2(player_x, 405.0), cpu);
    for center in [deck, drop, trigger] {
        let bounds = layout.rect(center, vec2(73.0, 100.0));
        draw_rectangle(bounds.x, bounds.y, bounds.w, bounds.h, POCKET);
        draw_rectangle_lines(bounds.x, bounds.y, bounds.w, bounds.h, 1.0, EDGE);
    }
    if !side.deck.is_empty() {
        draw_sleeve(layout.rect(deck, vec2(65.0, 94.0)), art.sleeve(cpu), cpu);
    }
    if let Some(card) = side.drop_zone.last() {
        draw_card_face(layout.rect(drop, vec2(65.0, 94.0)), card, false, card_art);
    }
    draw_count(
        layout,
        deck + vec2(0.0, if cpu { -58.0 } else { 58.0 }),
        &format!("DECK  {}", side.deck.len()),
    );
    draw_count(
        layout,
        drop + vec2(0.0, if cpu { -58.0 } else { 58.0 }),
        &format!("DROP  {}", side.drop_zone.len()),
    );
    layout.label(
        "TRIGGER",
        trigger + vec2(0.0, if cpu { -54.0 } else { 54.0 }),
        10.0,
        INK,
    );
    if let Some(check) = side.last_check.as_deref() {
        let bounds = layout.rect(trigger, vec2(58.0, 84.0));
        let name = super::checked_card_name(check);
        if let Some(texture) = card_art.get(&super::normalize_card_name(name)) {
            draw_sleeve(bounds, texture, cpu);
        } else {
            draw_wrapped(
                check,
                bounds.x + 3.0,
                bounds.y + 12.0 * layout.scale,
                (bounds.w - 6.0).max(1.0),
                (10.0 * layout.scale).max(8.0),
                INK,
                4,
            );
        }
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
        Color::from_rgba(9, 18, 31, 245),
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
    fn overlapping_damage_stays_in_one_pocket_and_each_exposed_strip_is_clickable() {
        for size in [
            vec2(MIN_WIDTH, TABLE_HEIGHT),
            vec2(1330.0, 662.0),
            vec2(744.0, 440.0),
        ] {
            let layout = BattlefieldLayout::new(Rect::new(0.0, 0.0, size.x, size.y));
            for cpu in [true, false] {
                let pocket = layout.rect(layout.damage_center(cpu), DAMAGE_POCKET);
                for count in 1..=6 {
                    for index in 0..count {
                        let card = layout.damage_card(index, cpu);
                        assert!(card.w > card.h, "damage cards must stay sideways");
                        let strip = vec2(
                            card.center().x,
                            if cpu {
                                card.bottom() - DAMAGE_STEP * layout.scale * 0.5
                            } else {
                                card.top() + DAMAGE_STEP * layout.scale * 0.5
                            },
                        );
                        assert_eq!(layout.damage_at(strip, count, cpu), Some(index));
                        assert_eq!(layout.damage_at(strip, 0, cpu), None);
                        assert_eq!(layout.damage_at(strip, count, !cpu), None);
                        assert!(pocket.contains(card.point()));
                        assert!(pocket.contains(vec2(card.right(), card.bottom())));
                        assert!(layout.bounds.contains(card.point()));
                        assert!(layout.bounds.contains(vec2(card.right(), card.bottom())));
                        if index + 1 < count {
                            assert!(card.overlaps(&layout.damage_card(index + 1, cpu)));
                        }
                    }
                    let top = layout.damage_card(count - 1, cpu);
                    assert_eq!(layout.damage_at(top.center(), count, cpu), Some(count - 1));
                }
            }
        }
    }

    #[test]
    fn guardian_accepts_cards_on_both_halves_of_the_complete_circle() {
        let layout = BattlefieldLayout::new(Rect::new(0.0, 0.0, MIN_WIDTH, TABLE_HEIGHT));
        let center = layout.guardian_center();
        for y in [-35.0, 35.0] {
            assert!(layout.guardian_at(layout.point(center + vec2(0.0, y))));
        }
        assert!(!layout.guardian_at(layout.point(center + vec2(0.0, 60.0))));
    }
}
