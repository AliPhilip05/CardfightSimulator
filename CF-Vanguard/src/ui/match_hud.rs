//! Desktop table frame, fighter status, and match controls.

use super::cards::fit_caption;
use super::drawing::{draw_centered, draw_wrapped, CREAM, CYAN, GOLD, MUTED, RED};
use crate::game::card::Player;
use crate::game::engine::{MatchGame, Phase};
use macroquad::prelude::*;

const CHROME: Color = Color::from_rgba(12, 22, 38, 255);
const SURFACE: Color = Color::from_rgba(18, 32, 51, 255);
const LINE: Color = Color::from_rgba(41, 60, 79, 255);
const DARK_TEXT: Color = Color::from_rgba(14, 23, 34, 255);

pub(super) struct MatchLayout {
    header: Rect,
    pub(super) board: Rect,
    pub(super) hand: Rect,
    pub(super) controls: Rect,
    pub(super) log: Rect,
}

impl MatchLayout {
    pub(super) fn new(width: f32, height: f32) -> Self {
        let gutter = (width * 0.028).clamp(12.0, 34.0);
        let inner_width = (width - 2.0 * gutter).max(1.0).min(1330.0);
        let left = (width - inner_width) * 0.5;
        let header_height = if width >= 810.0 { 66.0 } else { 92.0 };
        let header = Rect::new(left, 0.0, inner_width, header_height);
        let footer_height = if height >= 720.0 { 62.0 } else { 50.0 };
        let hand_height = (height * 0.176).clamp(96.0, 156.0);
        let controls = Rect::new(
            left,
            height - footer_height,
            inner_width,
            footer_height - 4.0,
        );
        let hand_width = inner_width.min(1020.0);
        let hand = Rect::new(
            (width - hand_width) * 0.5,
            controls.y - hand_height,
            hand_width,
            hand_height,
        );
        let board_top = header_height + 9.0;
        let board = Rect::new(
            left,
            board_top,
            inner_width,
            (hand.y - board_top - 5.0).max(1.0),
        );
        let log_width = (inner_width * 0.48).clamp(190.0, 356.0).min(inner_width);
        let log = Rect::new(
            board.right() - log_width - 8.0,
            board.y + 8.0,
            log_width,
            (board.h - 16.0).clamp(24.0, 318.0),
        );

        Self {
            header,
            board,
            hand,
            controls,
            log,
        }
    }
}

pub(super) fn draw_match_shell(layout: &MatchLayout) {
    let width = screen_width();
    let height = screen_height();
    draw_rectangle(0.0, 0.0, width, height, Color::from_rgba(9, 17, 29, 255));
    draw_rectangle(0.0, 0.0, width, layout.header.h, CHROME);
    draw_line(0.0, layout.header.h, width, layout.header.h, 1.0, LINE);

    let top = layout.hand.y + layout.hand.h * 0.31;
    draw_rectangle(0.0, top, width, height - top, CHROME);
    draw_line(0.0, top, width, top, 1.0, LINE);
    draw_line(
        layout.controls.x,
        layout.controls.y,
        layout.controls.right(),
        layout.controls.y,
        1.0,
        LINE,
    );
}

pub(super) fn draw_match_hud(game: &MatchGame, layout: &MatchLayout, log_open: bool) -> bool {
    let header = layout.header;
    let compact = header.h > 70.0;
    let phase_area = if compact {
        Rect::new(header.center().x - 121.0, 54.0, 242.0, 29.0)
    } else {
        Rect::new(header.center().x - 142.0, 18.0, 284.0, 31.0)
    };
    let column_width = if compact {
        header.w * 0.5 - 13.0
    } else {
        phase_area.x - header.x - 18.0
    };

    let player_y = if compact { 13.0 } else { 19.0 };
    draw_rectangle(header.x, player_y, 3.0, 31.0, CYAN);
    let title_size = if column_width < 220.0 { 14 } else { 17 };
    let title = fit_caption(&game.chapter.title, column_width - 12.0, title_size);
    draw_text(
        &title,
        header.x + 12.0,
        player_y + 11.0,
        title_size as f32,
        CREAM,
    );
    let line = format!(
        "YOU  ·  {}  ·  {}",
        fighter_counts(&game.player),
        if game.phase == Phase::Cpu {
            "Defending"
        } else {
            "Your turn"
        }
    );
    let line = fit_caption(&line, column_width - 12.0, 12);
    draw_text(&line, header.x + 12.0, player_y + 28.0, 12.0, MUTED);

    let rival_right = header.right() - 12.0;
    draw_rectangle(header.right() - 3.0, player_y, 3.0, 31.0, RED);
    let rival_name = fit_caption(&game.cpu.name, column_width - 12.0, title_size);
    draw_right(&rival_name, rival_right, player_y + 11.0, title_size, CREAM);
    let opponent_counts = fit_caption(&fighter_counts(&game.cpu), column_width - 12.0, 12);
    draw_right(&opponent_counts, rival_right, player_y + 28.0, 12, MUTED);

    let turn_width = if compact { 47.0 } else { 54.0 };
    draw_centered(
        &format!("T{:02}", game.turn),
        phase_area.x,
        phase_area.y + phase_area.h * 0.64,
        turn_width,
        13.0,
        MUTED,
    );
    let log_width = if compact { 38.0 } else { 43.0 };
    let phase_width = (phase_area.w - turn_width - log_width - 14.0) / 3.0;
    let active = match game.phase {
        Phase::Ride => Some(0),
        Phase::Call => Some(1),
        Phase::Battle | Phase::Cpu => Some(2),
        Phase::Finished => None,
    };
    for (index, label) in ["RIDE", "MAIN", "BATTLE"].into_iter().enumerate() {
        let x = phase_area.x + turn_width + index as f32 * (phase_width + 4.0);
        let slot = Rect::new(x, phase_area.y, phase_width, phase_area.h);
        if active == Some(index) {
            rounded_box(slot, 5.0, Color::from_rgba(53, 46, 34, 255));
            draw_line(
                slot.x + 8.0,
                slot.bottom() - 3.0,
                slot.right() - 8.0,
                slot.bottom() - 3.0,
                1.0,
                GOLD,
            );
        }
        draw_centered(
            label,
            slot.x,
            slot.y + slot.h * 0.63,
            slot.w,
            12.0,
            if active == Some(index) { GOLD } else { MUTED },
        );
    }
    let log = Rect::new(
        phase_area.right() - log_width,
        phase_area.y,
        log_width,
        phase_area.h,
    );
    draw_command_button("LOG", log, MUTED, log_open)
}

fn fighter_counts(player: &Player) -> String {
    format!(
        "{}/6 damage  ·  {} deck  ·  {} hand",
        player.damage.len(),
        player.deck.len(),
        player.hand.len(),
    )
}

pub(super) fn draw_battle_log(game: &MatchGame, rect: Rect) {
    rounded_box(
        Rect::new(rect.x + 2.0, rect.y + 6.0, rect.w, rect.h),
        8.0,
        Color::from_rgba(0, 0, 0, 110),
    );
    rounded_box(rect, 8.0, LINE);
    rounded_box(inset(rect, 1.0), 7.0, CHROME);
    draw_text("BATTLE LOG", rect.x + 16.0, rect.y + 24.0, 13.0, GOLD);
    draw_line(
        rect.x + 16.0,
        rect.y + 35.0,
        rect.right() - 16.0,
        rect.y + 35.0,
        1.0,
        LINE,
    );
    let lines = ((rect.h - 47.0) / 36.0).max(0.0) as usize;
    for (index, line) in game.log.iter().rev().take(lines).enumerate() {
        draw_wrapped(
            line,
            rect.x + 16.0,
            rect.y + 54.0 + index as f32 * 36.0,
            rect.w - 32.0,
            13.0,
            if index == 0 { CREAM } else { MUTED },
            2,
        );
    }
}

pub(super) fn draw_command_button(label: &str, rect: Rect, accent: Color, primary: bool) -> bool {
    let hovered = rect.contains(mouse_position().into());
    let fill = if primary {
        accent
    } else if hovered {
        Color::from_rgba(37, 54, 75, 255)
    } else {
        SURFACE
    };
    rounded_box(rect, 5.0, if primary || hovered { accent } else { LINE });
    rounded_box(inset(rect, 1.0), 4.0, fill);
    let font_size = if measure_text(label, None, 14, 1.0).width + 13.0 > rect.w {
        11.0
    } else {
        14.0
    };
    draw_centered(
        label,
        rect.x,
        rect.center().y + font_size * 0.34,
        rect.w,
        font_size,
        if primary {
            DARK_TEXT
        } else if hovered {
            CREAM
        } else {
            accent
        },
    );
    hovered && is_mouse_button_pressed(MouseButton::Left)
}

fn draw_right(text: &str, right: f32, y: f32, size: u16, color: Color) {
    let width = measure_text(text, None, size, 1.0).width;
    draw_text(text, right - width, y, size as f32, color);
}

fn inset(rect: Rect, amount: f32) -> Rect {
    Rect::new(
        rect.x + amount,
        rect.y + amount,
        rect.w - 2.0 * amount,
        rect.h - 2.0 * amount,
    )
}

fn rounded_box(rect: Rect, radius: f32, color: Color) {
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return;
    }
    let radius = radius.min(rect.h * 0.5).min(rect.w * 0.5);
    draw_rectangle(
        rect.x + radius,
        rect.y,
        rect.w - radius * 2.0,
        rect.h,
        color,
    );
    draw_rectangle(
        rect.x,
        rect.y + radius,
        rect.w,
        (rect.h - radius * 2.0).max(0.0),
        color,
    );
    for x in [rect.x + radius, rect.right() - radius] {
        for y in [rect.y + radius, rect.bottom() - radius] {
            draw_circle(x, y, radius, color);
        }
    }
}
