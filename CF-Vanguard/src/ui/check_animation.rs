//! Numbered, fighter-specific Drive/Damage cut-ins paced independently of game effects.

use super::art::normalize_card_name;
use super::board::{BattlefieldLayout, TabletopArt};
use super::cards::fit_caption;
use super::drawing::{draw_centered, CREAM, CYAN, GOLD, RED};
use super::fighters::FighterArt;
use super::motion::{interpolate_rect, smoothstep};
use crate::game::card::Trigger;
use crate::game::engine::{CheckKind, CheckReveal, MatchGame};
use macroquad::prelude::*;
use std::collections::HashMap;

const NORMAL_SECONDS: f64 = 1.55;
const TRIGGER_SECONDS: f64 = 2.65;
const ENTER_SECONDS: f32 = 0.24;
const FLIP_START: f32 = 0.34;
const FLIP_END: f32 = 0.76;
const TRIGGER_START: f32 = 0.90;

#[derive(Default)]
pub(super) struct CheckAnimation {
    sequence: u64,
    started: f64,
    reveal: Option<CheckReveal>,
}

impl CheckAnimation {
    pub(super) fn observe(&mut self, reveal: Option<&CheckReveal>, now: f64) {
        if let Some(reveal) = reveal.filter(|r| r.sequence != self.sequence) {
            self.sequence = reveal.sequence;
            self.started = now;
            self.reveal = Some(reveal.clone());
        }
    }

    pub(super) fn active(&self, now: f64) -> bool {
        self.reveal
            .as_ref()
            .is_some_and(|r| now - self.started < duration(r))
    }

    pub(super) fn revealed_name(&self, now: f64) -> Option<&str> {
        self.active(now)
            .then(|| self.reveal.as_ref().unwrap().card.name)
    }

    pub(super) fn draw(
        &self,
        game: &MatchGame,
        field: &BattlefieldLayout,
        tabletop: &TabletopArt,
        fighters: &FighterArt,
        textures: &HashMap<String, Texture2D>,
        now: f64,
    ) {
        let Some(reveal) = self.reveal.as_ref().filter(|_| self.active(now)) else {
            return;
        };
        let time = (now - self.started).max(0.0) as f32;
        let life = duration(reveal) as f32;
        let entrance = smoothstep(time / ENTER_SECONDS);
        let exit = smoothstep((time - life + 0.30) / 0.30);
        let alpha = entrance * (1.0 - exit);
        let width = screen_width();
        let height = screen_height();
        let scale = (width / 1440.0).min(height / 900.0).clamp(0.35, 1.8);
        let accent = if reveal.cpu { RED } else { CYAN };
        let fighter = checking_fighter(game, reveal.cpu);
        let mut frame = Rect::new(width * 0.045, height * 0.235, width * 0.91, height * 0.48);
        frame.x += (1.0 - entrance) * -width + exit * width;
        let slope = 27.0 * scale;

        draw_rectangle(
            0.0,
            0.0,
            width,
            height,
            Color::new(0.0, 0.01, 0.025, 0.66 * alpha),
        );
        cutin_frame(
            frame,
            slope,
            Color::new(0.015, 0.035, 0.04, alpha),
            accent.with_alpha(alpha),
        );
        draw_circuits(frame, scale, time, accent.with_alpha(alpha));
        draw_zone_art(frame, tabletop, scale, alpha);

        let sleeve = if reveal.cpu {
            &tabletop.kai_sleeve
        } else {
            &tabletop.aichi_sleeve
        };
        let deck = Rect::new(
            frame.x + frame.w * 0.10,
            frame.y + frame.h * 0.38,
            72.0 * scale,
            105.0 * scale,
        );
        for i in (0..4).rev() {
            draw_texture_ex(
                sleeve,
                deck.x + i as f32 * 2.5 * scale,
                deck.y - i as f32 * 2.5 * scale,
                WHITE.with_alpha(alpha),
                DrawTextureParams {
                    dest_size: Some(deck.size()),
                    rotation: -0.12,
                    ..Default::default()
                },
            );
        }
        draw_centered(
            "DECK",
            deck.x - 10.0 * scale,
            deck.bottom() + 24.0 * scale,
            deck.w + 20.0 * scale,
            13.0 * scale,
            CREAM.with_alpha(alpha),
        );

        let portrait_panel = Rect::new(
            frame.x + frame.w * 0.76,
            frame.y + 9.0 * scale,
            frame.w * 0.225,
            frame.h - 36.0 * scale,
        );
        draw_character(
            fighter,
            portrait_panel,
            fighters,
            accent,
            scale,
            alpha,
            smoothstep((time - 0.38) / 0.25),
        );

        let face_height = (frame.h * 0.78).min(330.0 * scale);
        let face = Rect::new(
            frame.x + frame.w * 0.39 - face_height * 0.344,
            frame.center().y - face_height * 0.50,
            face_height * 0.688,
            face_height,
        );
        let source = field.deck_rect(reveal.cpu);
        let motion = smoothstep((time - 0.14) / 0.36);
        let mut card = interpolate_rect(source, face, motion);
        if exit > 0.0 {
            card = interpolate_rect(face, field.check_rect(reveal.cpu), exit);
        }
        let flip = ((time - FLIP_START) / (FLIP_END - FLIP_START)).clamp(0.0, 1.0);
        let is_front = flip >= 0.5;
        let flip_width = (flip * std::f32::consts::PI).cos().abs().max(0.012);
        let glow = if reveal.trigger_active {
            trigger_color(reveal.card.trigger)
        } else {
            accent
        };
        if is_front {
            for ring in (1..=6).rev() {
                let margin = ring as f32 * 3.0 * scale;
                draw_rectangle_lines(
                    card.x - margin,
                    card.y - margin,
                    card.w + margin * 2.0,
                    card.h + margin * 2.0,
                    3.0 * scale,
                    glow.with_alpha(alpha * (0.18 - ring as f32 * 0.02)),
                );
            }
        }
        let image_rect = Rect::new(
            card.center().x - card.w * flip_width * 0.5,
            card.y,
            card.w * flip_width,
            card.h,
        );
        let texture = if is_front {
            textures.get(&normalize_card_name(reveal.card.name))
        } else {
            Some(sleeve)
        };
        if let Some(texture) = texture {
            draw_texture_ex(
                texture,
                image_rect.x,
                image_rect.y,
                WHITE.with_alpha(alpha),
                DrawTextureParams {
                    dest_size: Some(image_rect.size()),
                    ..Default::default()
                },
            );
        } else {
            draw_rectangle(
                image_rect.x,
                image_rect.y,
                image_rect.w,
                image_rect.h,
                Color::new(0.05, 0.10, 0.16, alpha),
            );
            if is_front {
                draw_centered(
                    "CARD REVEALED",
                    card.x,
                    card.center().y,
                    card.w,
                    13.0 * scale,
                    CREAM.with_alpha(alpha),
                );
            }
        }
        draw_rectangle_lines(
            image_rect.x - scale,
            image_rect.y - scale,
            image_rect.w + 2.0 * scale,
            image_rect.h + 2.0 * scale,
            1.5 * scale,
            WHITE.with_alpha(alpha * 0.8),
        );
        let flip_flash = (1.0 - ((time - 0.61) / 0.10).abs()).max(0.0) * 0.7;
        if flip_flash > 0.0 {
            draw_rectangle(
                card.x,
                card.y,
                card.w,
                card.h,
                WHITE.with_alpha(flip_flash * alpha),
            );
        }

        let title = if reveal.kind == CheckKind::Drive {
            "DRIVE CHECK"
        } else {
            "DAMAGE CHECK"
        };
        check_ribbon(title, frame, slope, scale, accent, alpha, time);
        if is_front {
            let name = fit_caption(reveal.card.name, frame.w * 0.57, (18.0 * scale) as u16);
            outlined_text(
                &name,
                vec2(frame.x + 24.0 * scale, frame.bottom() - 8.0 * scale),
                18.0 * scale,
                0.0,
                CREAM.with_alpha(alpha),
            );
        }
        if reveal.card.is_trigger() && time > TRIGGER_START {
            draw_trigger(reveal, frame, scale, time - TRIGGER_START, alpha);
        } else if is_front {
            outlined_text(
                "TRIGGER CHECK ZONE",
                vec2(frame.x + frame.w * 0.55, frame.y + frame.h * 0.52),
                18.0 * scale,
                -0.04,
                accent.with_alpha(alpha * 0.8),
            );
            if !reveal.card.is_trigger() {
                outlined_text(
                    "NO TRIGGER",
                    vec2(frame.x + frame.w * 0.55, frame.y + frame.h * 0.61),
                    26.0 * scale,
                    -0.04,
                    CREAM.with_alpha(alpha),
                );
            }
        }
    }
}

fn duration(reveal: &CheckReveal) -> f64 {
    if reveal.card.is_trigger() {
        TRIGGER_SECONDS
    } else {
        NORMAL_SECONDS
    }
}

fn checking_fighter(game: &MatchGame, cpu: bool) -> &str {
    if cpu {
        &game.cpu.name
    } else if game.player.name == "You" {
        "Aichi Sendou"
    } else {
        &game.player.name
    }
}

fn cutin_frame(rect: Rect, slope: f32, fill: Color, accent: Color) {
    let p = [
        vec2(rect.x + 10.0, rect.y + slope),
        vec2(rect.right(), rect.y - slope),
        vec2(rect.right() - 10.0, rect.bottom() - slope),
        vec2(rect.x, rect.bottom() + slope),
    ];
    quad(p, fill);
    for i in 0..4 {
        let a = p[i];
        let b = p[(i + 1) % 4];
        draw_line(a.x, a.y, b.x, b.y, 8.0, Color::new(0.0, 0.0, 0.0, accent.a));
        draw_line(
            a.x,
            a.y,
            b.x,
            b.y,
            3.0,
            Color::new(0.66, 0.73, 0.78, accent.a),
        );
        draw_line(a.x, a.y - 3.0, b.x, b.y - 3.0, 1.0, accent);
    }
}

fn quad(points: [Vec2; 4], color: Color) {
    draw_triangle(points[0], points[1], points[2], color);
    draw_triangle(points[0], points[2], points[3], color);
}

fn draw_circuits(frame: Rect, scale: f32, time: f32, accent: Color) {
    for row in 0..10 {
        let y = frame.y + 35.0 * scale + row as f32 * frame.h / 12.0;
        let x = frame.x + ((time * 115.0 + row as f32 * 79.0) % (frame.w * 0.66));
        let w = 72.0 * scale + row as f32 * 4.0 * scale;
        draw_line(
            x,
            y,
            x + w,
            y - 7.0 * scale,
            scale,
            accent.with_alpha(accent.a * 0.17),
        );
        draw_line(
            x + w,
            y - 7.0 * scale,
            x + w + 14.0 * scale,
            y - 21.0 * scale,
            scale,
            accent.with_alpha(accent.a * 0.22),
        );
    }
    for i in 0..7 {
        let x = frame.x + frame.w * (0.10 + i as f32 * 0.09);
        draw_poly_lines(
            x,
            frame.y + frame.h * 0.25,
            6,
            26.0 * scale,
            30.0,
            scale,
            Color::new(0.15, 0.4, 0.31, accent.a * 0.35),
        );
    }
}

fn draw_zone_art(frame: Rect, tabletop: &TabletopArt, scale: f32, alpha: f32) {
    for (fraction, diameter, source) in [
        (0.20, 102.0, Rect::new(370.0, 275.0, 282.0, 282.0)),
        (0.59, 132.0, Rect::new(690.0, 245.0, 350.0, 350.0)),
    ] {
        draw_texture_ex(
            &tabletop.zones,
            frame.x + frame.w * fraction,
            frame.y + 37.0 * scale,
            WHITE.with_alpha(alpha * 0.55),
            DrawTextureParams {
                source: Some(source),
                dest_size: Some(vec2(diameter * scale, diameter * 0.64 * scale)),
                ..Default::default()
            },
        );
    }
}

fn draw_character(
    name: &str,
    rect: Rect,
    fighters: &FighterArt,
    accent: Color,
    scale: f32,
    alpha: f32,
    entrance: f32,
) {
    let rect = Rect::new(rect.x + (1.0 - entrance) * rect.w, rect.y, rect.w, rect.h);
    let opacity = alpha * entrance;
    quad(
        [
            vec2(rect.x + 18.0 * scale, rect.y),
            vec2(rect.right(), rect.y - 18.0 * scale),
            vec2(rect.right(), rect.bottom()),
            vec2(rect.x - 12.0 * scale, rect.bottom() + 12.0 * scale),
        ],
        Color::new(0.035, 0.07, 0.16, opacity),
    );
    draw_line(
        rect.x + 18.0 * scale,
        rect.y,
        rect.x - 12.0 * scale,
        rect.bottom() + 12.0 * scale,
        3.0 * scale,
        accent.with_alpha(opacity),
    );
    if let Some(texture) = fighters.cutin(name) {
        // Upper-body crop of the unchanged official standing render.
        let source = Rect::new(24.0, 0.0, texture.width() - 48.0, texture.height() * 0.73);
        let h = rect.h - 30.0 * scale;
        let w = h * source.w / source.h;
        draw_texture_ex(
            texture,
            rect.center().x - w * 0.5,
            rect.y + 4.0 * scale,
            WHITE.with_alpha(opacity),
            DrawTextureParams {
                source: Some(source),
                dest_size: Some(vec2(w, h)),
                ..Default::default()
            },
        );
    } else if let Some(texture) = fighters.portrait(name) {
        draw_texture_ex(
            texture,
            rect.x + 15.0 * scale,
            rect.y + 12.0 * scale,
            WHITE.with_alpha(opacity),
            DrawTextureParams {
                dest_size: Some(vec2(rect.w - 30.0 * scale, rect.h - 45.0 * scale)),
                ..Default::default()
            },
        );
    } else {
        draw_centered(
            &name.chars().next().unwrap_or('?').to_string(),
            rect.x,
            rect.center().y,
            rect.w,
            72.0 * scale,
            accent.with_alpha(opacity),
        );
    }
    outlined_text(
        &fit_caption(name, rect.w - 12.0 * scale, (16.0 * scale) as u16),
        vec2(rect.x + 4.0 * scale, rect.bottom() - 3.0 * scale),
        16.0 * scale,
        0.0,
        CREAM.with_alpha(opacity),
    );
}

fn check_ribbon(
    title: &str,
    frame: Rect,
    slope: f32,
    scale: f32,
    accent: Color,
    alpha: f32,
    time: f32,
) {
    let top = vec2(frame.x + 20.0 * scale, frame.y + slope - 12.0 * scale);
    let bottom = vec2(
        frame.right() - 310.0 * scale,
        frame.bottom() - slope + 32.0 * scale,
    );
    for pos in [top, bottom] {
        let width = measure_text(title, None, (31.0 * scale) as u16, 1.0).width;
        quad(
            [
                pos + vec2(-16.0, -31.0) * scale,
                pos + vec2(width + 28.0 * scale, -31.0 * scale),
                pos + vec2(width + 10.0 * scale, 7.0 * scale),
                pos + vec2(-30.0, 7.0) * scale,
            ],
            Color::new(0.02, 0.025, 0.04, alpha),
        );
        outlined_text(
            title,
            pos,
            31.0 * scale,
            -0.035,
            Color::new(0.73, 0.84, 0.98, alpha),
        );
        draw_line(
            pos.x - 18.0 * scale,
            pos.y + 8.0 * scale,
            pos.x + width,
            pos.y - 2.0 * scale,
            2.0 * scale,
            accent.with_alpha(alpha),
        );
        let scan = (time * 1.3).fract();
        draw_line(
            pos.x + width * scan,
            pos.y - 25.0 * scale,
            pos.x + width * scan + 10.0 * scale,
            pos.y - 25.0 * scale,
            2.0 * scale,
            WHITE.with_alpha(alpha * 0.6),
        );
    }
}

fn trigger_color(trigger: Trigger) -> Color {
    match trigger {
        Trigger::Critical => Color::new(1.0, 0.78, 0.12, 1.0),
        Trigger::Draw => Color::new(1.0, 0.28, 0.36, 1.0),
        Trigger::Stand => Color::new(0.25, 0.75, 1.0, 1.0),
        Trigger::Heal => Color::new(0.28, 1.0, 0.55, 1.0),
        Trigger::None => CYAN,
    }
}

fn trigger_title(trigger: Trigger) -> &'static str {
    match trigger {
        Trigger::Critical => "CRITICAL TRIGGER",
        Trigger::Draw => "DRAW TRIGGER",
        Trigger::Stand => "STAND TRIGGER",
        Trigger::Heal => "HEAL TRIGGER",
        Trigger::None => "NO TRIGGER",
    }
}

fn secondary_label(reveal: &CheckReveal) -> &'static str {
    if !reveal.trigger_active {
        return "NO MATCHING CLAN";
    }
    match reveal.card.trigger {
        Trigger::Critical => "+1 CRITICAL",
        Trigger::Draw => "DRAW 1 CARD",
        Trigger::Stand if reveal.secondary_available => "STAND A REAR-GUARD",
        Trigger::Stand => "NO REAR-GUARD TO STAND",
        Trigger::Heal if reveal.secondary_available => "RECOVER 1 DAMAGE",
        Trigger::Heal => "HEAL CONDITION NOT MET",
        Trigger::None => "",
    }
}

fn draw_trigger(reveal: &CheckReveal, frame: Rect, scale: f32, time: f32, alpha: f32) {
    let pop = smoothstep(time / 0.15);
    let color = trigger_color(reveal.card.trigger);
    let origin = vec2(
        frame.x + frame.w * 0.53 + (1.0 - pop) * 100.0 * scale,
        frame.y + frame.h * 0.51,
    );
    let burst = (time / 0.6).clamp(0.0, 1.0);
    for i in 0..18 {
        let angle = i as f32 * std::f32::consts::TAU / 18.0 + time * 0.35;
        let from = origin + vec2(angle.cos(), angle.sin()) * (15.0 + burst * 35.0) * scale;
        let to = origin + vec2(angle.cos(), angle.sin()) * (40.0 + burst * 90.0) * scale;
        draw_line(
            from.x,
            from.y,
            to.x,
            to.y,
            1.5 * scale,
            color.with_alpha((1.0 - burst) * alpha * 0.7),
        );
    }
    let pulse = (1.0 - (time / 0.22).clamp(0.0, 1.0)) * 0.18 * alpha;
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        color.with_alpha(pulse),
    );
    let font = (35.0 * scale).min(frame.w * 0.25 / 11.0);
    outlined_text(
        "GET TRIGGER!",
        origin + vec2(0.0, -19.0 * scale),
        15.0 * scale,
        -0.04,
        CREAM.with_alpha(alpha),
    );
    outlined_text(
        trigger_title(reveal.card.trigger),
        origin + vec2(0.0, 18.0 * scale),
        font,
        -0.04,
        color.with_alpha(alpha),
    );
    outlined_text(
        if reveal.trigger_active {
            "+5000 POWER"
        } else {
            "TRIGGER UNIT"
        },
        origin + vec2(0.0, 53.0 * scale),
        24.0 * scale,
        -0.04,
        GOLD.with_alpha(alpha),
    );
    outlined_text(
        secondary_label(reveal),
        origin + vec2(0.0, 80.0 * scale),
        15.0 * scale,
        -0.04,
        CREAM.with_alpha(alpha),
    );
    trigger_symbol(
        reveal.card.trigger,
        origin + vec2(48.0, -83.0) * scale,
        28.0 * scale * (1.0 + (1.0 - pop) * 0.6),
        color.with_alpha(alpha),
    );
}

fn trigger_symbol(trigger: Trigger, center: Vec2, radius: f32, color: Color) {
    draw_circle_lines(
        center.x,
        center.y,
        radius * 1.14,
        radius * 0.04,
        color.with_alpha(color.a * 0.3),
    );
    draw_circle_lines(center.x, center.y, radius, radius * 0.06, color);
    match trigger {
        Trigger::Critical => {
            for i in 0..8 {
                let angle = i as f32 * std::f32::consts::TAU / 8.0;
                let a = center + Vec2::from_angle(angle) * radius * 0.84;
                let b = center + Vec2::from_angle(angle + 0.18) * radius * 0.31;
                let c = center + Vec2::from_angle(angle - 0.18) * radius * 0.31;
                draw_triangle(a, b, c, color);
            }
            draw_circle(center.x, center.y, radius * 0.19, color);
        }
        Trigger::Draw => {
            for i in 0..3 {
                draw_rectangle_lines(
                    center.x - radius * 0.50 + i as f32 * radius * 0.18,
                    center.y - radius * 0.61 + i as f32 * radius * 0.14,
                    radius * 0.67,
                    radius * 0.95,
                    radius * 0.08,
                    color,
                );
            }
        }
        Trigger::Stand => {
            draw_rectangle(
                center.x - radius * 0.14,
                center.y - radius * 0.15,
                radius * 0.28,
                radius * 0.78,
                color,
            );
            draw_triangle(
                center + vec2(0.0, -radius * 0.72),
                center + vec2(-radius * 0.57, -radius * 0.03),
                center + vec2(radius * 0.57, -radius * 0.03),
                color,
            );
        }
        Trigger::Heal => {
            draw_rectangle(
                center.x - radius * 0.17,
                center.y - radius * 0.65,
                radius * 0.34,
                radius * 1.30,
                color,
            );
            draw_rectangle(
                center.x - radius * 0.65,
                center.y - radius * 0.17,
                radius * 1.30,
                radius * 0.34,
                color,
            );
        }
        Trigger::None => {}
    }
}

fn outlined_text(text: &str, position: Vec2, size: f32, rotation: f32, color: Color) {
    for delta in [
        vec2(-1.5, 0.0),
        vec2(1.5, 0.0),
        vec2(0.0, -1.5),
        vec2(0.0, 1.5),
    ] {
        draw_text_ex(
            text,
            position.x + delta.x,
            position.y + delta.y,
            TextParams {
                font_size: size as u16,
                rotation,
                color: BLACK.with_alpha(color.a),
                ..Default::default()
            },
        );
    }
    draw_text_ex(
        text,
        position.x,
        position.y,
        TextParams {
            font_size: size as u16,
            rotation,
            color,
            ..Default::default()
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::card::Card;

    fn reveal(sequence: u64, trigger: Trigger) -> CheckReveal {
        CheckReveal {
            sequence,
            kind: CheckKind::Drive,
            cpu: false,
            card: Card::trigger("Repeated card", "Royal Paladin", trigger),
            trigger_active: trigger != Trigger::None,
            secondary_available: true,
        }
    }

    #[test]
    fn identical_consecutive_checks_replay_once_per_sequence() {
        let mut animation = CheckAnimation::default();
        animation.observe(Some(&reveal(1, Trigger::Critical)), 10.0);
        assert!(animation.active(11.0));
        animation.observe(Some(&reveal(1, Trigger::Critical)), 12.0);
        assert!(!animation.active(13.0));
        animation.observe(Some(&reveal(2, Trigger::Critical)), 13.0);
        assert!(animation.active(13.0));
        assert_eq!(animation.started, 13.0);
    }

    #[test]
    fn reveals_hold_resolution_until_the_full_cutin_finishes() {
        let mut animation = CheckAnimation::default();
        animation.observe(Some(&reveal(1, Trigger::None)), 0.0);
        assert!(animation.active(NORMAL_SECONDS - 0.001));
        assert!(!animation.active(NORMAL_SECONDS));
        animation.observe(Some(&reveal(2, Trigger::Heal)), 10.0);
        assert!(animation.active(10.0 + TRIGGER_SECONDS - 0.001));
        assert!(!animation.active(10.0 + TRIGGER_SECONDS));
    }

    #[test]
    fn trigger_copy_respects_clan_and_secondary_conditions() {
        let mut event = reveal(1, Trigger::Heal);
        event.secondary_available = false;
        assert_eq!(secondary_label(&event), "HEAL CONDITION NOT MET");
        event.trigger_active = false;
        assert_eq!(secondary_label(&event), "NO MATCHING CLAN");
        for kind in [
            Trigger::Critical,
            Trigger::Draw,
            Trigger::Stand,
            Trigger::Heal,
        ] {
            assert!(trigger_title(kind).ends_with("TRIGGER"));
        }
    }

    #[test]
    fn cutin_identity_uses_the_checking_fighter_not_the_attacker() {
        let mut game = MatchGame::new(crate::story::CHAPTERS[3]);
        assert_eq!(checking_fighter(&game, false), "Aichi Sendou");
        assert_eq!(checking_fighter(&game, true), "Ren Suzugamori");
        game.player.name = "Toshiki Kai".into();
        game.cpu.name = "Reversed Asaka Narumi".into();
        assert_eq!(checking_fighter(&game, false), "Toshiki Kai");
        assert_eq!(checking_fighter(&game, true), "Reversed Asaka Narumi");
    }
}
