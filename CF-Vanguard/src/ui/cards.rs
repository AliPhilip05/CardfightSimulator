//! Card faces and hand placement.

use super::drawing::{
    button, clan_color, draw_centered, draw_round_panel, draw_star, draw_wrapped, CREAM, CYAN,
    GOLD, MUTED, PANEL, PANEL_LIGHT,
};
use crate::game::card::{Card, Unit};
use crate::game::engine::{MatchGame, Phase};
use macroquad::prelude::*;
use std::collections::HashMap;

pub(super) fn draw_field_unit(
    rect: Rect,
    unit: &Unit,
    card_art: &HashMap<String, Texture2D>,
    stand_progress: f32,
) {
    let mut face = if unit.rested {
        Rect::new(
            rect.center().x - rect.h * 0.5,
            rect.center().y - rect.w * 0.5,
            rect.h,
            rect.w,
        )
    } else {
        rect
    };
    if unit.rested && !unit.locked && stand_progress > 0.0 {
        let progress = super::motion::smoothstep(stand_progress);
        let angle = std::f32::consts::FRAC_PI_2 * (1.0 - progress);
        let size = vec2(
            rect.w * angle.cos() + rect.h * angle.sin(),
            rect.h * angle.cos() + rect.w * angle.sin(),
        );
        face = Rect::new(
            rect.center().x - size.x * 0.5,
            rect.center().y - size.y * 0.5,
            size.x,
            size.y,
        );
        if let Some(texture) = card_art.get(&super::normalize_card_name(unit.card.name)) {
            draw_texture_ex(
                texture,
                rect.x,
                rect.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(rect.size()),
                    rotation: angle,
                    ..Default::default()
                },
            );
            draw_rectangle_lines(
                rect.x - 2.0,
                rect.y - 2.0,
                rect.w + 4.0,
                rect.h + 4.0,
                2.0,
                Color {
                    a: (std::f32::consts::PI * progress).sin() * 0.8,
                    ..CYAN
                },
            );
        } else {
            draw_card_face(rect, &unit.card, false, card_art);
        }
    } else {
        draw_card_face(face, &unit.card, unit.rested, card_art);
    }
    if unit.locked {
        draw_rectangle(
            face.x,
            face.y,
            face.w,
            face.h,
            Color::new(0.02, 0.03, 0.05, 0.90),
        );
        draw_centered("LOCKED", face.x, face.center().y + 4.0, face.w, 13.0, CYAN);
        return;
    }
    let h = (rect.h * 0.14).max(14.0);
    let w = rect.w * 1.12;
    let x = face.center().x - w * 0.5;
    let y = face.bottom() - h * 0.35;
    draw_rectangle(x, y, w, h, Color::from_rgba(3, 9, 15, 236));
    let power = unit.power().to_string();
    let size = (rect.h * 0.16).max(14.0);
    draw_centered(&power, x + 1.0, y + h * 0.86 + 1.0, w, size, BLACK);
    draw_centered(
        &power,
        x,
        y + h * 0.86,
        w,
        size,
        if unit.power_bonus > 0 {
            CYAN
        } else {
            Color::from_rgba(255, 222, 101, 255)
        },
    );
    if unit.critical() > 1 {
        draw_text(
            &format!("CRIT {}", unit.critical()),
            face.x,
            face.y - 8.0,
            13.0,
            GOLD,
        );
    }
    let grade = format!("G{}", unit.card.grade);
    draw_rectangle(
        face.x - 3.0,
        face.y - 3.0,
        22.0,
        15.0,
        Color::from_rgba(7, 18, 27, 235),
    );
    draw_text(&grade, face.x, face.y + 9.0, 12.0, WHITE);
}

pub(super) fn draw_guard_card(rect: Rect, card: &Card, card_art: &HashMap<String, Texture2D>) {
    draw_card_face(rect, card, true, card_art);
}

pub(super) fn draw_card_face(
    rect: Rect,
    card: &Card,
    turned: bool,
    card_art: &HashMap<String, Texture2D>,
) {
    let accent = clan_color(card.clan);
    draw_rectangle(
        rect.x + 2.0,
        rect.y + 3.0,
        rect.w,
        rect.h,
        Color::new(0.0, 0.0, 0.0, 0.36),
    );
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL);
    if let Some(texture) = card_art.get(&super::normalize_card_name(card.name)) {
        let size = if turned {
            vec2(rect.h, rect.w)
        } else {
            vec2(rect.w, rect.h)
        };
        let top_left = rect.center() - size * 0.5;
        draw_texture_ex(
            texture,
            top_left.x,
            top_left.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                rotation: if turned {
                    std::f32::consts::FRAC_PI_2
                } else {
                    0.0
                },
                ..Default::default()
            },
        );
    } else {
        draw_rectangle(
            rect.x + 2.0,
            rect.y + 2.0,
            rect.w - 4.0,
            rect.h - 4.0,
            PANEL_LIGHT,
        );
        draw_clan_art(
            card.clan,
            rect.center().x,
            rect.y + rect.h * 0.38,
            rect.w.min(rect.h) * 0.21,
            accent,
        );
        draw_wrapped(
            card.name,
            rect.x + 4.0,
            rect.y + rect.h * 0.72,
            rect.w - 8.0,
            9.0,
            CREAM,
            2,
        );
    }
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, accent);
}

fn draw_clan_art(clan: &str, center_x: f32, center_y: f32, radius: f32, accent: Color) {
    draw_circle(
        center_x,
        center_y,
        radius + 8.0,
        Color::new(accent.r, accent.g, accent.b, 0.12),
    );
    draw_circle_lines(
        center_x,
        center_y,
        radius + 6.0,
        1.0,
        Color::new(accent.r, accent.g, accent.b, 0.65),
    );

    match clan {
        "Kagero" => {
            draw_triangle(
                vec2(center_x - radius * 0.72, center_y - radius * 0.85),
                vec2(center_x - radius * 0.22, center_y - radius * 0.14),
                vec2(center_x - radius * 0.77, center_y + radius * 0.12),
                CREAM,
            );
            draw_triangle(
                vec2(center_x + radius * 0.72, center_y - radius * 0.85),
                vec2(center_x + radius * 0.22, center_y - radius * 0.14),
                vec2(center_x + radius * 0.77, center_y + radius * 0.12),
                CREAM,
            );
            draw_triangle(
                vec2(center_x, center_y - radius * 0.35),
                vec2(center_x - radius * 0.58, center_y + radius * 0.52),
                vec2(center_x + radius * 0.58, center_y + radius * 0.52),
                accent,
            );
            draw_line(
                center_x - radius * 0.35,
                center_y + radius * 0.12,
                center_x + radius * 0.35,
                center_y + radius * 0.12,
                2.0,
                CREAM,
            );
        }
        "Narukami" => {
            let points = [
                (-0.12, -0.95),
                (-0.65, -0.12),
                (-0.10, -0.12),
                (-0.42, 0.88),
                (0.67, -0.30),
                (0.12, -0.30),
            ];
            for pair in points.windows(2) {
                draw_line(
                    center_x + pair[0].0 * radius,
                    center_y + pair[0].1 * radius,
                    center_x + pair[1].0 * radius,
                    center_y + pair[1].1 * radius,
                    3.0,
                    CREAM,
                );
            }
            draw_circle(center_x, center_y, radius * 0.14, accent);
        }
        "Link Joker" => {
            draw_circle(
                center_x,
                center_y,
                radius * 0.62,
                Color::new(0.04, 0.035, 0.11, 0.92),
            );
            draw_circle_lines(center_x, center_y, radius * 0.82, 2.0, CREAM);
            draw_star(center_x, center_y, radius * 0.48, accent);
            draw_circle(center_x, center_y, radius * 0.12, CREAM);
        }
        "Royal Paladin" | "Gold Paladin" => {
            draw_circle_lines(center_x, center_y, radius * 0.68, 2.0, CREAM);
            draw_line(
                center_x - radius * 0.58,
                center_y - radius * 0.48,
                center_x + radius * 0.58,
                center_y + radius * 0.48,
                3.0,
                CREAM,
            );
            draw_line(
                center_x + radius * 0.58,
                center_y - radius * 0.48,
                center_x - radius * 0.58,
                center_y + radius * 0.48,
                3.0,
                CREAM,
            );
            draw_star(center_x, center_y, radius * 0.34, accent);
        }
        "Aqua Force" => {
            for row in 0..3 {
                let wave_y = center_y - radius * 0.35 + row as f32 * radius * 0.36;
                let mut previous: Option<Vec2> = None;
                for step in 0..=12 {
                    let offset = -radius * 0.8 + step as f32 * radius * 0.13;
                    let point = vec2(
                        center_x + offset,
                        wave_y + (offset * 0.22).sin() * radius * 0.18,
                    );
                    if let Some(from) = previous {
                        draw_line(from.x, from.y, point.x, point.y, 2.2, CREAM);
                    }
                    previous = Some(point);
                }
            }
        }
        _ => draw_star(center_x, center_y, radius * 0.72, CREAM),
    }
}

const HAND_HEADER: f32 = 31.0;
const HAND_GUTTER: f32 = 16.0;
const CARD_ASPECT: f32 = 0.68;

pub(super) struct HandLayout {
    panel: Rect,
    pub page: usize,
    pages: usize,
    capacity: usize,
    card_size: Vec2,
    card_step: f32,
    first_x: f32,
    count: usize,
}

impl HandLayout {
    pub(super) fn new(panel: Rect, count: usize, requested_page: usize) -> Self {
        let available = (panel.w - HAND_GUTTER * 2.0).max(1.0);
        let height = (panel.h - HAND_HEADER - 8.0)
            .clamp(1.0, 150.0)
            .min(available / CARD_ASPECT);
        let card_size = vec2(height * CARD_ASPECT, height);
        let min_step = card_size.x * 0.56;
        let capacity = (1 + ((available - card_size.x) / min_step).floor() as usize).min(20);
        let pages = count.div_ceil(capacity).max(1);
        let page = requested_page.min(pages - 1);
        let visible = (count - page * capacity).min(capacity);
        let intervals = visible.saturating_sub(1) as f32;
        let card_step = ((available - card_size.x) / intervals.max(1.0)).min(card_size.x * 0.91);
        let row_width = card_size.x + intervals * card_step;
        Self {
            panel,
            page,
            pages,
            capacity,
            card_size,
            card_step,
            first_x: panel.center().x - row_width * 0.5,
            count,
        }
    }

    pub(super) fn during_draw(panel: Rect, count: usize, page: usize, progress: f32) -> Self {
        let mut current = Self::new(panel, count, page);
        let target = Self::new(panel, count + 1, current.page);
        current.first_x += (target.first_x - current.first_x) * progress;
        current.card_step += (target.card_step - current.card_step) * progress;
        current
    }

    pub(super) fn draw_destination(&self) -> Option<Rect> {
        let target = Self::new(self.panel, self.count + 1, self.page);
        target
            .visible_indices()
            .contains(&self.count)
            .then(|| target.card_rect(self.count))
    }

    pub(super) fn next_page_button(&self) -> Rect {
        Rect::new(
            self.panel.right() - HAND_GUTTER - 22.0,
            self.panel.y + 4.0,
            22.0,
            21.0,
        )
    }

    fn visible_indices(&self) -> std::ops::Range<usize> {
        let first = self.page * self.capacity;
        first..(first + self.capacity).min(self.count)
    }

    fn card_rect(&self, index: usize) -> Rect {
        let position = (index - self.page * self.capacity) as f32;
        Rect::new(
            self.first_x + position * self.card_step,
            self.panel.y + HAND_HEADER,
            self.card_size.x,
            self.card_size.y,
        )
    }

    fn exposed_width(&self, index: usize) -> f32 {
        if index + 1 == self.visible_indices().end {
            self.card_size.x
        } else {
            self.card_step
        }
    }

    pub(super) fn card_at(&self, point: Vec2) -> Option<usize> {
        // Later cards paint over the right edge of earlier ones.
        self.visible_indices()
            .rev()
            .find(|&index| self.card_rect(index).contains(point))
    }

    pub(super) fn draw_page_controls(&self, interactive: bool) -> Option<usize> {
        if self.pages <= 1 {
            return None;
        }
        let x = self.panel.right() - HAND_GUTTER;
        let y = self.panel.y + 4.0;
        let previous = self.page > 0 && button("<", x - 108.0, y, 22.0, 21.0, MUTED);
        let next_rect = self.next_page_button();
        let next = self.page + 1 < self.pages
            && button(
                ">",
                next_rect.x,
                next_rect.y,
                next_rect.w,
                next_rect.h,
                MUTED,
            );
        draw_centered(
            &format!("{} / {}", self.page + 1, self.pages),
            x - 83.0,
            y + 15.0,
            58.0,
            12.0,
            CREAM,
        );

        if !interactive {
            return None;
        }
        let mut wheel = 0.0;
        if self.panel.contains(mouse_position().into()) && !is_mouse_button_down(MouseButton::Left)
        {
            wheel = mouse_wheel().1;
        }
        if previous || (wheel > 0.0 && self.page > 0) {
            Some(self.page - 1)
        } else if next || (wheel < 0.0 && self.page + 1 < self.pages) {
            Some(self.page + 1)
        } else {
            None
        }
    }
}

pub(super) fn draw_player_hand(
    game: &MatchGame,
    layout: &HandLayout,
    selected: Option<usize>,
    card_art: &HashMap<String, Texture2D>,
) {
    let panel = layout.panel;
    let edge = Color::new(0.23, 0.37, 0.53, 0.85);
    // Cards float over the arena, with a quiet caption instead of a boxed tray.
    let hovered = if game.phase == Phase::Draw {
        None
    } else {
        layout.card_at(mouse_position().into())
    };
    let active = hovered.or(selected);
    draw_text(
        &format!("HAND  {}", layout.count),
        panel.x + HAND_GUTTER,
        panel.y + 19.0,
        15.0,
        CREAM,
    );
    if let Some(card) = active.and_then(|index| game.player.hand.get(index)) {
        let caption_x = panel.x + 102.0;
        let caption_width =
            panel.right() - caption_x - if layout.pages > 1 { 134.0 } else { HAND_GUTTER };
        if caption_width > 0.0 {
            let caption = fit_caption(&hand_card_caption(card), caption_width, 14);
            draw_text(&caption, caption_x, panel.y + 19.0, 14.0, MUTED);
        }
    }

    let ride_options = if game.phase == Phase::Ride {
        game.ride_options()
    } else {
        Vec::new()
    };
    let call_options = if game.phase == Phase::Call {
        game.call_options()
    } else {
        Vec::new()
    };
    for index in layout.visible_indices() {
        let rect = layout.card_rect(index);
        let card = &game.player.hand[index];
        let can_guard = game.can_guard(index);
        let accent = if selected == Some(index) {
            GOLD
        } else if can_guard || ride_options.contains(&index) || call_options.contains(&index) {
            CYAN
        } else {
            edge
        };
        draw_card_face(rect, card, false, card_art);
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, accent);
        if can_guard && layout.exposed_width(index) >= 18.0 {
            draw_shield_label(rect, layout.exposed_width(index), card.shield, accent);
        }
    }

    if let Some(index) = hovered {
        let rect = layout.card_rect(index);
        let width = layout.exposed_width(index);
        let hover_color = if selected == Some(index) { GOLD } else { CREAM };
        draw_line(rect.x, rect.y, rect.x + width, rect.y, 2.0, hover_color);
        draw_line(rect.x, rect.y, rect.x, rect.bottom(), 2.0, hover_color);
        if !is_mouse_button_down(MouseButton::Left) {
            draw_hand_preview(panel, &game.player.hand[index], card_art);
        }
    }

    if layout.count == 0 {
        draw_centered(
            "No cards in hand",
            panel.x,
            panel.center().y + 15.0,
            panel.w,
            15.0,
            MUTED,
        );
    }
}

fn hand_card_caption(card: &Card) -> String {
    format!(
        "{}  ·  Grade {}  ·  {} power  ·  {} shield",
        card.name, card.grade, card.power, card.shield
    )
}

fn draw_shield_label(rect: Rect, visible_width: f32, shield: i32, color: Color) {
    let label = format!("+{shield}");
    let fontsize = ((visible_width - 3.0) / label.len() as f32 * 1.45).clamp(7.0, 11.0);
    let height = fontsize + 5.0;
    draw_rectangle(
        rect.x + 1.0,
        rect.bottom() - height - 1.0,
        visible_width - 2.0,
        height,
        PANEL,
    );
    draw_centered(
        &label,
        rect.x,
        rect.bottom() - 4.0,
        visible_width,
        fontsize,
        color,
    );
}

fn draw_hand_preview(panel: Rect, card: &Card, card_art: &HashMap<String, Texture2D>) {
    let height = (panel.top() - 76.0).min(226.0);
    if height < 106.0 || panel.w < 245.0 {
        return;
    }
    let width = height * CARD_ASPECT;
    let x = panel.right() - width - HAND_GUTTER;
    let y = panel.top() - height - 11.0;
    let rect = Rect::new(x, y, width, height);
    draw_round_panel(x - 4.0, y - 4.0, width + 8.0, height + 8.0, PANEL, MUTED);
    draw_card_face(rect, card, false, card_art);
}

pub(super) fn fit_caption(text: &str, width: f32, font_size: u16) -> String {
    if measure_text(text, None, font_size, 1.0).width <= width {
        return text.to_owned();
    }
    let mut caption = String::new();
    let ellipsis_width = measure_text("…", None, font_size, 1.0).width;
    if ellipsis_width > width {
        return caption;
    }
    for word in text.split_whitespace() {
        let previous_length = caption.len();
        if previous_length > 0 {
            caption.push(' ');
        }
        caption.push_str(word);
        if measure_text(&caption, None, font_size, 1.0).width + ellipsis_width > width {
            caption.truncate(previous_length);
            break;
        }
    }
    caption.push('…');
    caption
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_hand_cards_use_the_visible_card_for_input() {
        let panel = Rect::new(250.0, 620.0, 440.0, 144.0);
        let layout = HandLayout::new(panel, 7, 0);
        let first = layout.card_rect(0);
        let second = layout.card_rect(1);
        let y = first.center().y;
        let overlap = vec2(second.left() + 1.0, y);

        assert!(first.contains(overlap));
        assert_eq!(layout.card_at(overlap), Some(1));
        assert_eq!(layout.card_at(vec2(first.left() + 1.0, y)), Some(0));
        assert_eq!(layout.card_at(vec2(panel.x + 1.0, y)), None);
    }

    #[test]
    fn every_hand_card_remains_reachable_when_the_ribbon_pages() {
        let panel = Rect::new(20.0, 450.0, 460.0, 156.0);
        let first_page = HandLayout::new(panel, 41, 0);
        assert!(first_page.pages > 1);
        let mut reachable = Vec::new();
        for page in 0..first_page.pages {
            let layout = HandLayout::new(panel, 41, page);
            for index in layout.visible_indices() {
                let rect = layout.card_rect(index);
                let point = vec2(rect.x + 1.0, rect.center().y);
                assert_eq!(layout.card_at(point), Some(index));
                assert!(rect.right() <= panel.right());
                reachable.push(index);
            }
        }
        assert_eq!(reachable, (0..41).collect::<Vec<_>>());
        assert_eq!(HandLayout::new(panel, 2, first_page.pages).page, 0);
    }
    #[test]
    fn draw_reserves_the_same_slot_the_final_hand_uses() {
        let panel = Rect::new(300.0, 700.0, 850.0, 180.0);
        for count in [0, 5, 8, 12] {
            let old = HandLayout::new(panel, count, 0);
            let reserved = HandLayout::during_draw(panel, count, 0, 1.0);
            let final_hand = HandLayout::new(panel, count + 1, 0);
            for i in old.visible_indices() {
                if final_hand.visible_indices().contains(&i) {
                    assert_eq!(reserved.card_rect(i), final_hand.card_rect(i));
                }
            }
            assert_eq!(
                reserved.draw_destination(),
                final_hand
                    .visible_indices()
                    .contains(&count)
                    .then(|| final_hand.card_rect(count))
            );
        }
        let full_page = HandLayout::new(panel, 0, 0).capacity;
        assert!(HandLayout::new(panel, full_page, 0)
            .draw_destination()
            .is_none());
        assert!(HandLayout::new(panel, full_page + 1, 0)
            .draw_destination()
            .is_none());
        assert!(HandLayout::new(panel, full_page + 1, 1)
            .draw_destination()
            .is_some());
    }
}
