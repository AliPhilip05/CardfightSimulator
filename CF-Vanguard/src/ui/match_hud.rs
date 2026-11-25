//! Field-first fight overlays: phase badge, corner fighters, prompts, and compact commands.

use super::cards::fit_caption;
use super::drawing::{draw_centered, CREAM, CYAN, GOLD, MUTED, RED};
use super::match_flow::MatchPrompt;
use super::MatchAction;
use crate::game::card::Player;
use crate::game::engine::{MatchGame, Phase, TurnStage};
use macroquad::prelude::*;

const CHROME: Color = Color::from_rgba(8, 14, 22, 240);
const SURFACE: Color = Color::from_rgba(16, 27, 39, 232);
const LINE: Color = Color::from_rgba(78, 111, 135, 180);
const DARK_TEXT: Color = Color::from_rgba(8, 16, 23, 255);

pub(super) struct MatchLayout {
    header: Rect,
    pub(super) phases: Rect,
    pub(super) decision: Rect,
    pub(super) board: Rect,
    pub(super) hand: Rect,
    pub(super) actions: Rect,
    pub(super) log: Rect,
    pub(super) power: Rect,
    utilities: Rect,
    player_status: Rect,
    opponent_status: Rect,
}

impl MatchLayout {
    pub(super) fn new(width: f32, height: f32) -> Self {
        let compact = width < 1050.0;
        let side = if compact { 166.0 } else { 194.0 };
        let phase_w = if compact { 156.0 } else { 194.0 };
        let status_w = (width * 0.245).clamp(220.0, 348.0);
        let header = Rect::new(12.0, 10.0, phase_w, 26.0);
        let phases = Rect::new(12.0, 40.0, phase_w, 150.0);
        let opponent_status = Rect::new(width - status_w - 12.0, 10.0, status_w, 65.0);
        let player_status = Rect::new(12.0, height - 80.0, status_w, 68.0);
        let decision = Rect::new(12.0, phases.bottom() + 4.0, phase_w, 66.0);
        let hand_h = (height * 0.19).clamp(118.0, 180.0);
        let board = Rect::new(
            70.0,
            20.0,
            (width - side - 88.0).max(1.0),
            (height - hand_h - 34.0).max(1.0),
        );
        let hand_x = player_status.right().min(board.w * 0.29);
        let hand = Rect::new(
            hand_x,
            height - hand_h - 6.0,
            (width - side - hand_x - 14.0).max(1.0),
            hand_h,
        );
        let actions = Rect::new(
            width - side - 12.0,
            (height * 0.39).max(125.0),
            side,
            (height * 0.61 - 78.0).max(1.0),
        );
        let utilities = Rect::new(width - side - 12.0, height - 46.0, side, 32.0);
        let log = Rect::new(
            (width - 414.0).max(10.0),
            height - 320.0,
            400.0_f32.min(width - 20.0),
            258.0,
        );
        Self {
            power: Rect::new(actions.x, 112.0, actions.w, 115.0),
            header,
            phases,
            decision,
            board,
            hand,
            actions,
            log,
            utilities,
            player_status,
            opponent_status,
        }
    }

    pub(super) fn blocks_field_input(&self, point: Vec2) -> bool {
        [
            self.header,
            self.phases,
            self.decision,
            self.actions,
            self.utilities,
            self.player_status,
            self.opponent_status,
        ]
        .iter()
        .any(|r| r.contains(point))
    }
}

pub(super) fn draw_match_shell(_layout: &MatchLayout) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(3, 9, 12, 255),
    );
    // Quiet hexagons extend behind the arena and floating overlays.
    let r = 26.0;
    for row in 0..(screen_height() / 45.0) as i32 + 2 {
        for col in 0..(screen_width() / 52.0) as i32 + 2 {
            let x = col as f32 * 51.0 + if row % 2 == 0 { 0.0 } else { 25.5 };
            draw_poly(
                x,
                row as f32 * 44.0,
                6,
                r - 2.0,
                30.0,
                Color::from_rgba(10, 25, 22, 255),
            );
        }
    }
}

pub(super) fn draw_match_hud(
    game: &MatchGame,
    layout: &MatchLayout,
    aichi: Option<&Texture2D>,
    opponent: Option<&Texture2D>,
) {
    let cpu = game.phase == Phase::Cpu;
    let accent = if cpu { RED } else { CYAN };
    let turn = game.turn;
    let r = layout.header;
    draw_text(
        &format!(
            "{}  {turn}",
            if game.phase == Phase::Setup {
                "PREPARING FIGHT"
            } else if cpu {
                "OPPONENT'S TURN"
            } else {
                "YOUR TURN"
            }
        ),
        r.x + 4.0,
        r.y + 19.0,
        20.0,
        accent,
    );
    fighter_bar(&game.player, layout.player_status, aichi, CYAN, false);
    fighter_bar(&game.cpu, layout.opponent_status, opponent, RED, true);
}

fn fighter_bar(
    player: &Player,
    r: Rect,
    portrait: Option<&Texture2D>,
    accent: Color,
    opponent: bool,
) {
    let portrait_x = if opponent {
        r.right() - 59.0
    } else {
        r.x + 1.0
    };
    let text_x = if opponent { r.x + 9.0 } else { r.x + 69.0 };
    beveled_panel(r, CHROME, accent);
    draw_circle(portrait_x + 29.0, r.center().y, 30.0, accent);
    draw_circle(portrait_x + 29.0, r.center().y, 27.0, SURFACE);
    if let Some(image) = portrait {
        draw_texture_ex(
            image,
            portrait_x + 4.0,
            r.y + 4.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(51.0, 58.0)),
                ..Default::default()
            },
        );
    } else {
        draw_centered(
            &player.name.chars().next().unwrap_or('R').to_string(),
            portrait_x + 3.0,
            r.y + 44.0,
            52.0,
            30.0,
            CREAM,
        );
    }
    let width = r.w - 80.0;
    draw_text(
        &fit_caption(&player.name, width, 17),
        text_x,
        r.y + 21.0,
        17.0,
        CREAM,
    );
    draw_text(
        &format!("DAMAGE  {} / 6", player.damage.len()),
        text_x,
        r.y + 40.0,
        15.0,
        accent,
    );
    draw_resource_count(vec2(text_x + 5.0, r.y + 52.0), player.deck.len(), false);
    draw_resource_count(vec2(text_x + 65.0, r.y + 52.0), player.hand.len(), true);
    draw_text(
        &format!("CB {}", player.face_up_damage()),
        text_x + 116.0,
        r.y + 57.0,
        13.0,
        MUTED,
    );
}

/// Draws a deck stack or fanned hand icon next to its count.
fn draw_resource_count(origin: Vec2, count: usize, hand: bool) {
    for i in 0..3 {
        let x = origin.x + i as f32 * if hand { 4.0 } else { 2.0 };
        let y = origin.y - i as f32 * if hand { 0.8 } else { 1.5 };
        draw_rectangle(x, y - 5.0, 8.0, 11.0, CHROME);
        draw_rectangle_lines(x, y - 5.0, 8.0, 11.0, 1.0, CREAM);
    }
    draw_text(
        &count.to_string(),
        origin.x + 22.0,
        origin.y + 5.0,
        17.0,
        CREAM,
    );
}

pub(super) fn draw_phase_track(game: &MatchGame, rect: Rect) {
    for (i, stage) in TurnStage::ALL.into_iter().enumerate() {
        let active = Some(stage) == game.stage();
        let y = rect.y + 19.0 + i as f32 * 24.0;
        if active {
            draw_rectangle(
                rect.x,
                y - 20.0,
                rect.w,
                25.0,
                Color::from_rgba(5, 12, 15, 185),
            );
            draw_rectangle(
                rect.x,
                y - 20.0,
                3.0,
                25.0,
                if game.phase == Phase::Cpu { RED } else { CYAN },
            );
        }
        draw_text(
            stage.label(),
            rect.x + 9.0,
            y,
            if active { 28.0 } else { 18.0 },
            if active {
                WHITE
            } else {
                Color::from_rgba(136, 148, 153, 190)
            },
        );
        if active {
            // A second close pass gives the active label weight without another typeface.
            draw_text(stage.label(), rect.x + 9.7, y, 28.0, WHITE);
        }
    }
}

pub(super) fn draw_decision_banner(prompt: &MatchPrompt, rect: Rect) {
    // Only decisions need a short instruction. Ordinary phases are explained by the phase list.
    if prompt.tag != "YOUR DECISION"
        || prompt.step.ends_with("PHASE")
        || prompt.step == "BATTLE PHASE / ATTACK"
    {
        return;
    }
    draw_left_lines(
        &prompt.title,
        rect.x + 8.0,
        rect.y + 16.0,
        rect.w - 12.0,
        15.0,
        prompt.accent,
        3,
    );
}

#[derive(Default)]
pub(super) struct PowerDisplay {
    attack: f32,
    defense: f32,
}

impl PowerDisplay {
    /// Animates the current attack and defense totals after boosts, guards, and triggers.
    pub(super) fn draw(&mut self, totals: Option<(i32, i32, i32)>, r: Rect) {
        let (attack, defense, shield) = totals.unwrap_or((0, 0, 0));
        let blend = (get_frame_time() * 12.0).clamp(0.0, 1.0);
        self.attack += (attack as f32 - self.attack) * blend;
        self.defense += ((defense + shield) as f32 - self.defense) * blend;
        if totals.is_none() && self.attack < 30.0 && self.defense < 30.0 {
            return;
        }
        beveled_panel(r, Color::from_rgba(5, 12, 17, 210), LINE);
        let ceiling = self.attack.max(self.defense).max(30000.0) * 1.08;
        for (i, label, value, animated, color) in [
            (0, "ATTACK", attack, self.attack, GOLD),
            (1, "DEFENSE", defense + shield, self.defense, CYAN),
        ] {
            let y = r.y + 20.0 + i as f32 * 47.0;
            draw_text(label, r.x + 10.0, y, 12.0, MUTED);
            draw_text(&value.to_string(), r.x + 10.0, y + 24.0, 27.0, color);
            draw_rectangle(r.x + 10.0, y + 28.0, r.w - 20.0, 4.0, SURFACE);
            draw_rectangle(
                r.x + 10.0,
                y + 28.0,
                (r.w - 20.0) * (animated / ceiling).clamp(0.0, 1.0),
                4.0,
                color,
            );
        }
    }
}

pub(super) fn draw_action_panel(
    game: &MatchGame,
    prompt: &MatchPrompt,
    rect: Rect,
    log_open: bool,
    selecting_attack: bool,
) -> Option<MatchAction> {
    let mut chosen = None;
    let mut y = rect.y;
    let h = if rect.w < 180.0 { 34.0 } else { 42.0 };
    if !prompt.choices.is_empty() {
        draw_text(
            if game.pending_attack.is_some() {
                "BATTLE DECISION"
            } else {
                "YOUR MOVE"
            },
            rect.x + 2.0,
            y,
            11.0,
            prompt.accent,
        );
        y += 12.0;
    }
    for choice in &prompt.choices {
        if draw_command_button(
            choice.label,
            Rect::new(rect.x, y, rect.w, h),
            prompt.accent,
            choice.primary,
        ) {
            chosen = Some(choice.action);
        }
        y += h + 7.0;
    }
    for (label, enabled, action) in [
        (
            "LIMIT BREAK · CB 2",
            !selecting_attack && game.can_limit_break(),
            MatchAction::LimitBreak,
        ),
        (
            "RE-STAND · DISCARD 3",
            !selecting_attack && game.can_restand(),
            MatchAction::Restand,
        ),
    ] {
        if enabled {
            y += 8.0;
            if draw_command_button(label, Rect::new(rect.x, y, rect.w, h), GOLD, false) {
                chosen = Some(action);
            }
            y += h;
        }
    }
    if prompt.choices.is_empty()
        && prompt.tag != "YOUR DECISION"
        && (game.phase == Phase::Cpu || game.pending_attack.is_some())
    {
        draw_text(
            if game.pending_attack.is_some() {
                "RESOLVING CHECKS"
            } else if matches!(game.phase, Phase::Stand | Phase::Draw) {
                ""
            } else {
                "OPPONENT PLAYING"
            },
            rect.x + 4.0,
            y + 15.0,
            12.0,
            RED,
        );
        for i in 0..3 {
            draw_circle(
                rect.x + 12.0 + i as f32 * 12.0,
                y + 33.0,
                2.0,
                Color::new(
                    RED.r,
                    RED.g,
                    RED.b,
                    0.25 + 0.75 * ((get_time() as f32 * 3.0 - i as f32).sin() * 0.5 + 0.5),
                ),
            );
        }
    }
    let utility_y = screen_height() - 46.0;
    let w = (rect.w - 7.0) * 0.5;
    if draw_command_button(
        if log_open { "CLOSE LOG" } else { "LOG" },
        Rect::new(rect.x, utility_y, w, 32.0),
        MUTED,
        false,
    ) {
        chosen = Some(MatchAction::ToggleLog);
    }
    if draw_command_button(
        "MENU",
        Rect::new(rect.x + w + 7.0, utility_y, w, 32.0),
        MUTED,
        false,
    ) {
        chosen = Some(MatchAction::Menu);
    }
    chosen
}

pub(super) fn draw_battle_log(game: &MatchGame, rect: Rect) {
    beveled_panel(rect, Color::from_rgba(5, 13, 21, 246), LINE);
    draw_text("RECENT PLAYS", rect.x + 14.0, rect.y + 24.0, 15.0, CREAM);
    draw_text(
        "Esc to close",
        rect.right() - 88.0,
        rect.y + 23.0,
        11.0,
        MUTED,
    );
    draw_line(
        rect.x + 14.0,
        rect.y + 34.0,
        rect.right() - 14.0,
        rect.y + 34.0,
        1.0,
        LINE,
    );
    let rows = ((rect.h - 45.0) / 26.0) as usize;
    for (i, text) in game.log.iter().rev().take(rows).enumerate() {
        draw_text(
            &fit_caption(text, rect.w - 30.0, 13),
            rect.x + 14.0,
            rect.y + 54.0 + i as f32 * 26.0,
            13.0,
            if i == 0 { CYAN } else { MUTED },
        );
    }
}

pub(super) fn draw_command_button(label: &str, rect: Rect, accent: Color, primary: bool) -> bool {
    let hover = rect.contains(mouse_position().into());
    beveled_panel(
        rect,
        if primary {
            accent
        } else if hover {
            Color::from_rgba(28, 50, 65, 250)
        } else {
            SURFACE
        },
        if hover || primary { accent } else { LINE },
    );
    draw_centered(
        &fit_caption(label, rect.w - 12.0, 14),
        rect.x,
        rect.center().y + 5.0,
        rect.w,
        14.0,
        if primary { DARK_TEXT } else { CREAM },
    );
    hover && is_mouse_button_pressed(MouseButton::Left)
}

fn beveled_panel(r: Rect, fill: Color, edge: Color) {
    let c = 9.0_f32.min(r.h * 0.25);
    let p = [
        vec2(r.x + c, r.y),
        vec2(r.right(), r.y),
        vec2(r.right(), r.bottom() - c),
        vec2(r.right() - c, r.bottom()),
        vec2(r.x, r.bottom()),
        vec2(r.x, r.y + c),
    ];
    for i in 0..6 {
        draw_triangle(r.center(), p[i], p[(i + 1) % 6], fill);
        draw_line(
            p[i].x,
            p[i].y,
            p[(i + 1) % 6].x,
            p[(i + 1) % 6].y,
            1.0,
            edge,
        );
    }
}

fn draw_left_lines(
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    size: f32,
    color: Color,
    max_lines: usize,
) {
    let mut line = String::new();
    let mut row = 0;
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.into()
        } else {
            format!("{line} {word}")
        };
        if measure_text(&candidate, None, size as u16, 1.0).width > width && !line.is_empty() {
            draw_text(&line, x, y + row as f32 * (size + 3.0), size, color);
            row += 1;
            if row >= max_lines {
                return;
            }
            line = word.into();
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        draw_text(&line, x, y + row as f32 * (size + 3.0), size, color);
    }
}
