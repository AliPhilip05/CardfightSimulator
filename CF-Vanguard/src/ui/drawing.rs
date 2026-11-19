//! Shared palette and lightweight vector drawing primitives.

use macroquad::prelude::*;

pub(super) const INK: Color = Color::new(0.035, 0.055, 0.105, 1.0);
pub(super) const PANEL: Color = Color::new(0.065, 0.095, 0.16, 1.0);
pub(super) const PANEL_LIGHT: Color = Color::new(0.10, 0.14, 0.22, 1.0);
pub(super) const GOLD: Color = Color::new(0.88, 0.69, 0.34, 1.0);
pub(super) const CREAM: Color = Color::new(0.94, 0.91, 0.82, 1.0);
pub(super) const MUTED: Color = Color::new(0.61, 0.68, 0.79, 1.0);
pub(super) const CYAN: Color = Color::new(0.32, 0.78, 0.88, 1.0);
pub(super) const RED: Color = Color::new(0.89, 0.35, 0.38, 1.0);

pub(super) fn draw_ambient_background() {
    let width = screen_width();
    let height = screen_height();
    for index in 0..8 {
        let alpha = 0.035 + index as f32 * 0.006;
        draw_circle(
            width * 0.83,
            height * 0.18,
            95.0 + index as f32 * 27.0,
            Color::new(0.17, 0.31, 0.53, alpha),
        );
    }
    draw_line(
        0.0,
        height * 0.42,
        width,
        height * 0.42,
        1.0,
        Color::new(0.65, 0.55, 0.33, 0.11),
    );
}

pub(super) fn draw_header(title: &str, subtitle: &str) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        94.0,
        Color::new(0.025, 0.04, 0.08, 0.96),
    );
    draw_text(title, 52.0, 43.0, 29.0, CREAM);
    draw_text(subtitle, 54.0, 72.0, 15.0, MUTED);
    draw_line(
        30.0,
        94.0,
        screen_width() - 30.0,
        94.0,
        1.0,
        Color::new(GOLD.r, GOLD.g, GOLD.b, 0.55),
    );
}

pub(super) fn button(label: &str, x: f32, y: f32, w: f32, h: f32, accent: Color) -> bool {
    let rect = Rect::new(x, y, w, h);
    let hover = rect.contains(vec2(mouse_position().0, mouse_position().1));
    let fill = if hover {
        Color::new(accent.r * 0.32, accent.g * 0.28, accent.b * 0.18, 1.0)
    } else {
        PANEL
    };
    draw_round_panel(x, y, w, h, fill, accent);
    draw_centered(
        label,
        x,
        y + h * 0.59,
        w,
        15.0,
        if hover { CREAM } else { accent },
    );
    hover && is_mouse_button_pressed(MouseButton::Left)
}

pub(super) fn draw_round_panel(x: f32, y: f32, w: f32, h: f32, fill: Color, border: Color) {
    draw_rectangle(x + 4.0, y + 4.0, w, h, Color::new(0.0, 0.0, 0.0, 0.25));
    draw_rectangle(x, y, w, h, border);
    draw_rectangle(x + 1.5, y + 1.5, w - 3.0, h - 3.0, fill);
}

pub(super) fn draw_centered(text: &str, x: f32, y: f32, width: f32, size: f32, color: Color) {
    let dimensions = measure_text(text, None, size as u16, 1.0);
    draw_text(text, x + (width - dimensions.width) * 0.5, y, size, color);
}

pub(super) fn draw_wrapped(
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    size: f32,
    color: Color,
    max_lines: usize,
) {
    let mut line = String::new();
    let mut lines = Vec::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if measure_text(&candidate, None, size as u16, 1.0).width > width && !line.is_empty() {
            lines.push(line);
            line = word.to_string();
            if lines.len() >= max_lines {
                break;
            }
        } else {
            line = candidate;
        }
    }
    if lines.len() < max_lines && !line.is_empty() {
        lines.push(line);
    }
    for (index, line) in lines.iter().take(max_lines).enumerate() {
        draw_centered(line, x, y + index as f32 * (size + 2.0), width, size, color);
    }
}

pub(super) fn draw_star_emblem(x: f32, y: f32, radius: f32) {
    draw_circle(
        x,
        y,
        radius + 20.0,
        Color::new(GOLD.r, GOLD.g, GOLD.b, 0.08),
    );
    draw_circle_lines(x, y, radius + 8.0, 2.0, GOLD);
    draw_circle_lines(
        x,
        y,
        radius - 5.0,
        1.0,
        Color::new(GOLD.r, GOLD.g, GOLD.b, 0.55),
    );
    draw_star(x, y, radius, CREAM);
}

pub(super) fn draw_star(x: f32, y: f32, radius: f32, color: Color) {
    for index in 0..8 {
        let angle = index as f32 * std::f32::consts::PI / 4.0;
        let end_x = x + angle.cos() * radius;
        let end_y = y + angle.sin() * radius;
        draw_line(
            x,
            y,
            end_x,
            end_y,
            if index % 2 == 0 { 2.0 } else { 1.0 },
            color,
        );
    }
}

pub(super) fn draw_school_crest(x: f32, y: f32, color: Color, style: usize) {
    draw_circle_lines(x, y, 42.0, 2.0, color);
    draw_circle_lines(x, y, 31.0, 1.0, Color::new(color.r, color.g, color.b, 0.5));
    match style {
        0 => {
            draw_line(x - 18.0, y - 20.0, x + 18.0, y + 20.0, 4.0, color);
            draw_line(x + 18.0, y - 20.0, x - 18.0, y + 20.0, 4.0, color);
        }
        1 => {
            draw_star(x, y, 28.0, color);
        }
        _ => {
            draw_line(x - 22.0, y, x + 22.0, y, 3.0, color);
            draw_line(x, y - 22.0, x, y + 22.0, 3.0, color);
            draw_circle(x, y, 8.0, color);
        }
    }
}

pub(super) fn clan_color(clan: &str) -> Color {
    match clan {
        "Royal Paladin" | "Gold Paladin" => Color::new(0.82, 0.68, 0.35, 1.0),
        "Kagero" | "Narukami" => Color::new(0.78, 0.34, 0.31, 1.0),
        "Link Joker" => Color::new(0.49, 0.39, 0.78, 1.0),
        "Aqua Force" => Color::new(0.27, 0.65, 0.83, 1.0),
        "Shadow Paladin" | "Pale Moon" => Color::new(0.58, 0.40, 0.72, 1.0),
        _ => Color::new(0.44, 0.68, 0.53, 1.0),
    }
}
