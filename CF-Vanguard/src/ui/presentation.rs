//! The shared opening, title, menu and loading presentation.

use super::embedded_texture;
use macroquad::prelude::*;

const BLUE: Color = Color::from_rgba(101, 218, 255, 255);
const GOLD: Color = Color::from_rgba(255, 212, 115, 255);
const WHITE_INK: Color = Color::from_rgba(229, 242, 255, 255);

pub(super) struct PresentationArt {
    cray: Texture2D,
    aichi: Texture2D,
    kai: Texture2D,
    logo: Texture2D,
    font: Font,
}

impl PresentationArt {
    pub fn new() -> Self {
        Self {
            cray: embedded_texture(include_bytes!("../../assets/presentation/planet-cray.png")),
            aichi: embedded_texture(include_bytes!("../../assets/fighters/aichi-standing.png")),
            kai: embedded_texture(include_bytes!("../../assets/fighters/kai-standing.png")),
            logo: embedded_texture(include_bytes!(
                "../../assets/presentation/vanguard-logo.png"
            )),
            font: load_ttf_font_from_bytes(include_bytes!(
                "../../assets/fonts/Rajdhani-SemiBold.ttf"
            ))
            .expect("bundled title font must decode"),
        }
    }

    pub fn font(&self) -> Font {
        self.font.clone()
    }

    pub fn scale(&self) -> f32 {
        (screen_width() / 1440.0)
            .min(screen_height() / 900.0)
            .clamp(0.45, 1.5)
    }

    pub fn text(&self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: Some(&self.font),
                font_size: size.max(10.0) as u16,
                color,
                ..Default::default()
            },
        );
    }

    fn centered(&self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        let width = measure_text(text, Some(&self.font), size.max(10.0) as u16, 1.0).width;
        self.text(text, x - width * 0.5, y, size, color);
    }

    pub fn scene(&self) {
        let w = screen_width();
        let h = screen_height();
        let zoom = (w / self.cray.width()).max(h / self.cray.height());
        let size = self.cray.size() * zoom;
        draw_texture_ex(
            &self.cray,
            (w - size.x) * 0.5,
            (h - size.y) * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
        gradient(
            Rect::new(w * 0.40, 0.0, w * 0.60, h),
            [
                Color::new(0.012, 0.032, 0.085, 0.0),
                Color::new(0.012, 0.032, 0.085, 0.78),
                Color::new(0.012, 0.032, 0.085, 0.78),
                Color::new(0.012, 0.032, 0.085, 0.0),
            ],
        );
        gradient(
            Rect::new(0.0, h * 0.72, w, h * 0.28),
            [
                Color::new(0.012, 0.025, 0.06, 0.0),
                Color::new(0.012, 0.025, 0.06, 0.0),
                Color::new(0.012, 0.025, 0.06, 0.88),
                Color::new(0.012, 0.025, 0.06, 0.88),
            ],
        );
        // Sample only the nonempty source bounds; preserve the downloaded PNGs.
        let character_height = (h * 0.82).min(w * 0.61);
        let bob = (get_time() as f32 * 0.65).sin() * 2.0;
        self.character(
            &self.kai,
            Rect::new(98.0, 32.0, 314.0, 592.0),
            vec2(w * 0.045, h - character_height + bob),
            character_height,
        );
        self.character(
            &self.aichi,
            Rect::new(110.0, 20.0, 236.0, 604.0),
            vec2(w * 0.265, h - character_height * 0.92 - bob),
            character_height * 0.92,
        );
        for index in 0..28 {
            let t = get_time() as f32;
            let px = ((index as f32 * 89.3 + t * (3.0 + (index % 3) as f32)) % w).max(0.0);
            let py = (index as f32 * 71.7 - t * 12.0).rem_euclid(h);
            let glow = 0.20 + ((index as f32 + t).sin() * 0.15);
            draw_circle(
                px,
                py,
                if index % 4 == 0 { 2.0 } else { 1.0 },
                Color::new(0.6, 0.87, 1.0, glow),
            );
        }
        let s = self.scale();
        draw_line(
            28.0 * s,
            h - 33.0 * s,
            w - 28.0 * s,
            h - 33.0 * s,
            1.0,
            Color::new(0.4, 0.75, 1.0, 0.36),
        );
        self.text(
            "A FAN-MADE CARDFIGHT EXPERIENCE",
            30.0 * s,
            h - 13.0 * s,
            13.0 * s,
            WHITE_INK,
        );
    }

    fn character(&self, texture: &Texture2D, source: Rect, pos: Vec2, height: f32) {
        draw_texture_ex(
            texture,
            pos.x,
            pos.y,
            WHITE,
            DrawTextureParams {
                source: Some(source),
                dest_size: Some(vec2(height * source.w / source.h, height)),
                ..Default::default()
            },
        );
    }

    pub fn heading(&self) {
        let w = screen_width();
        let h = screen_height();
        let s = self.scale();
        let size = vec2(w * 0.40, w * 0.40 * self.logo.height() / self.logo.width());
        let origin = vec2(w * 0.545, h * 0.10);
        draw_texture_ex(
            &self.logo,
            origin.x + 3.0 * s,
            origin.y + 5.0 * s,
            Color::new(0.0, 0.0, 0.0, 0.55),
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
        draw_texture_ex(
            &self.logo,
            origin.x,
            origin.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
        let bottom = origin.y + size.y;
        self.centered(
            "LIMIT BREAK  /  BREAK RIDE",
            w * 0.745,
            bottom + 25.0 * s,
            24.0 * s,
            GOLD,
        );
        self.centered(
            "STAND UP, VANGUARD!",
            w * 0.745,
            bottom + 53.0 * s,
            18.0 * s,
            BLUE,
        );
    }

    pub fn title(&self) -> bool {
        self.scene();
        self.heading();
        let s = self.scale();
        let pulse = 0.65 + (get_time() as f32 * 2.4).sin() * 0.3;
        self.centered(
            "PRESS ENTER OR CLICK TO START",
            screen_width() * 0.745,
            screen_height() * 0.72,
            23.0 * s,
            Color {
                a: pulse,
                ..WHITE_INK
            },
        );
        is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left)
    }

    pub fn menu_button(&self, label: &str, detail: &str, index: usize) -> bool {
        let s = self.scale();
        let rect = Rect::new(
            screen_width() * 0.565,
            screen_height() * 0.54 + index as f32 * 76.0 * s,
            screen_width() * 0.36,
            62.0 * s,
        );
        let hovered = rect.contains(mouse_position().into());
        let accent = if hovered { GOLD } else { BLUE };
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.018, 0.05, 0.11, if hovered { 0.96 } else { 0.86 }),
        );
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            1.0,
            Color { a: 0.7, ..accent },
        );
        draw_rectangle(rect.x, rect.y, 4.0 * s, rect.h, accent);
        draw_triangle(
            vec2(rect.x + 18.0 * s, rect.center().y - 6.0 * s),
            vec2(rect.x + 18.0 * s, rect.center().y + 6.0 * s),
            vec2(rect.x + 25.0 * s, rect.center().y),
            accent,
        );
        self.text(
            label,
            rect.x + 39.0 * s,
            rect.y + 27.0 * s,
            25.0 * s,
            WHITE_INK,
        );
        self.text(
            detail,
            rect.x + 40.0 * s,
            rect.y + 47.0 * s,
            14.0 * s,
            Color::from_rgba(150, 189, 218, 255),
        );
        hovered && is_mouse_button_pressed(MouseButton::Left)
    }

    pub fn loading(&self, completed: f32, status: &str) {
        self.scene();
        self.heading();
        let s = self.scale();
        let left = screen_width() * 0.565;
        let width = screen_width() * 0.36;
        let y = screen_height() * 0.63;
        self.text(status, left, y - 19.0 * s, 23.0 * s, WHITE_INK);
        draw_rectangle(left, y, width, 7.0 * s, Color::from_rgba(19, 51, 79, 255));
        draw_rectangle(left, y, width * completed.clamp(0.0, 1.0), 7.0 * s, BLUE);
        self.text(
            &format!("{:03}%", (completed * 100.0) as u32),
            left,
            y + 30.0 * s,
            18.0 * s,
            BLUE,
        );
        self.text(
            "Preparing your next cardfight",
            left,
            y + 62.0 * s,
            16.0 * s,
            WHITE_INK,
        );
        // The spinner animates while the percentage represents completed work.
        let center = vec2(left + width - 15.0 * s, y + 26.0 * s);
        for index in 0..8 {
            let angle = index as f32 * std::f32::consts::TAU / 8.0 + get_time() as f32 * 2.0;
            let pos = center + vec2(angle.cos(), angle.sin()) * 8.0 * s;
            draw_circle(
                pos.x,
                pos.y,
                1.6 * s,
                Color {
                    a: (index + 1) as f32 / 8.0,
                    ..BLUE
                },
            );
        }
    }
}

// A single interpolated quad avoids seams from overlapping translucent strips.
fn gradient(rect: Rect, colors: [Color; 4]) {
    draw_mesh(&Mesh {
        vertices: vec![
            Vertex::new(rect.x, rect.y, 0.0, 0.0, 0.0, colors[0]),
            Vertex::new(rect.right(), rect.y, 0.0, 1.0, 0.0, colors[1]),
            Vertex::new(rect.right(), rect.bottom(), 0.0, 1.0, 1.0, colors[2]),
            Vertex::new(rect.x, rect.bottom(), 0.0, 0.0, 1.0, colors[3]),
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: None,
    });
}
