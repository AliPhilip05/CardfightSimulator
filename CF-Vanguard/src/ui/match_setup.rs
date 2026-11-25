//! Pre-fight presentation inspired by Dear Days' paired hand choices and 3DS redraw screen.
use super::cards::draw_card_face;
use super::match_hud::draw_command_button;
use super::*;

impl App {
    pub(super) fn draw_setup(&mut self) {
        let Some(game) = self.match_game.as_mut() else {
            return;
        };
        self.card_art.preload(
            &game
                .player
                .hand
                .iter()
                .map(|c| c.name.to_string())
                .collect::<Vec<_>>(),
        );
        let layout = MatchLayout::new(screen_width(), screen_height());
        draw_match_shell(&layout);
        draw_match_hud(
            game,
            &layout,
            self.fighters.portrait("Aichi"),
            self.fighters.portrait(&game.cpu.name),
        );
        let cx = screen_width() * 0.5;
        let cy = screen_height() * 0.48;
        let scale = (screen_width() / 1440.0)
            .min(screen_height() / 960.0)
            .clamp(0.6, 1.4);
        let stage = game.setup_stage;
        match stage {
            SetupStage::RockPaperScissors | SetupStage::Reveal => {
                setup_heading(
                    "ROCK  /  PAPER  /  SCISSORS",
                    "Win to choose first or second",
                    scale,
                );
                draw_centered(
                    &game.cpu.name,
                    cx - 350.0 * scale,
                    cy - 185.0 * scale,
                    700.0 * scale,
                    24.0 * scale,
                    RED,
                );
                let revealed = stage == SetupStage::Reveal;
                for (i, sign) in RpsChoice::ALL.into_iter().enumerate() {
                    let x = cx + (i as f32 - 1.0) * 190.0 * scale;
                    let upper = Rect::new(
                        x - 76.0 * scale,
                        cy - 155.0 * scale,
                        152.0 * scale,
                        125.0 * scale,
                    );
                    let lower = Rect::new(
                        x - 76.0 * scale,
                        cy + 85.0 * scale,
                        152.0 * scale,
                        125.0 * scale,
                    );
                    let (human, cpu) = game.rps_result.unwrap_or((sign, sign));
                    sign_tile(upper, sign, RED, revealed && cpu == sign, false);
                    if sign_tile(lower, sign, CYAN, revealed && human == sign, !revealed) {
                        let cpu = RpsChoice::ALL[rand::gen_range(0, 3) as usize];
                        game.play_rps(sign, cpu);
                        self.cpu_continue_at = get_time() + 1.25;
                    }
                }
                let result = if revealed {
                    match game.player_won_rps {
                        None => "DRAW — TRY AGAIN",
                        Some(true) => "YOU WIN",
                        Some(false) => "OPPONENT WINS — CHOOSES FIRST",
                    }
                } else {
                    "CHOOSE YOUR HAND"
                };
                draw_centered(
                    result,
                    cx - 400.0 * scale,
                    cy + 26.0 * scale,
                    800.0 * scale,
                    29.0 * scale,
                    GOLD,
                );
                if revealed && get_time() >= self.cpu_continue_at {
                    game.finish_rps_reveal();
                }
            }
            SetupStage::ChooseOrder => {
                setup_heading("YOU WON", "Choose your place in the turn order", scale);
                for (i, label, first) in [(0, "GO FIRST", true), (1, "GO SECOND", false)] {
                    let r = Rect::new(
                        cx - 180.0 * scale,
                        cy - 90.0 * scale + i as f32 * 115.0 * scale,
                        360.0 * scale,
                        85.0 * scale,
                    );
                    if draw_command_button(label, r, if first { CYAN } else { GOLD }, false) {
                        game.choose_turn_order(first);
                    }
                }
            }
            SetupStage::Mulligan | SetupStage::HandsReady => {
                let choosing = stage == SetupStage::Mulligan;
                setup_heading(
                    if choosing {
                        "CHOOSE YOUR OPENING HAND"
                    } else {
                        "BOTH FIGHTERS READY"
                    },
                    if choosing {
                        "Select up to 5 cards to replace. Replacements are drawn before shuffling."
                    } else {
                        "Your opening hand is set. Stand up your Vanguard when ready."
                    },
                    scale,
                );
                let card_w = 145.0 * scale;
                let card_h = card_w * 1.45;
                let gap = 18.0 * scale;
                let left = cx - (card_w * 5.0 + gap * 4.0) * 0.5;
                for i in 0..5 {
                    let back = Rect::new(
                        cx - 145.0 * scale + i as f32 * 62.0 * scale,
                        cy - 188.0 * scale,
                        52.0 * scale,
                        75.0 * scale,
                    );
                    draw_texture_ex(
                        &self.tabletop_art.kai_sleeve,
                        back.x,
                        back.y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(back.size()),
                            ..Default::default()
                        },
                    );
                }
                draw_centered(
                    &format!(
                        "{} · {}",
                        game.cpu.name,
                        if choosing { "WAITING" } else { "READY" }
                    ),
                    cx - 350.0 * scale,
                    cy - 200.0 * scale,
                    700.0 * scale,
                    18.0 * scale,
                    RED,
                );
                for (i, card) in game.player.hand.iter().take(5).enumerate() {
                    let selected = self.mulligan_selection.contains(&i);
                    let r = Rect::new(
                        left + i as f32 * (card_w + gap),
                        cy - 60.0 * scale - if selected { 14.0 * scale } else { 0.0 },
                        card_w,
                        card_h,
                    );
                    draw_card_face(r, card, false, self.card_art.textures());
                    if selected {
                        draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(5, 12, 20, 100));
                        draw_rectangle_lines(r.x - 3.0, r.y - 3.0, r.w + 6.0, r.h + 6.0, 3.0, GOLD);
                        draw_centered("RETURN", r.x, r.center().y + 7.0, r.w, 22.0 * scale, GOLD);
                    }
                    if choosing
                        && r.contains(mouse_position().into())
                        && is_mouse_button_pressed(MouseButton::Left)
                    {
                        if selected {
                            self.mulligan_selection.retain(|&n| n != i);
                        } else {
                            self.mulligan_selection.push(i);
                        }
                    }
                }
                let r = Rect::new(
                    cx - 170.0 * scale,
                    cy + 200.0 * scale,
                    340.0 * scale,
                    55.0 * scale,
                );
                let label = if choosing {
                    if self.mulligan_selection.is_empty() {
                        "KEEP HAND".into()
                    } else {
                        format!("REDRAW {} CARDS", self.mulligan_selection.len())
                    }
                } else {
                    "STAND UP, VANGUARD!".into()
                };
                if draw_command_button(&label, r, GOLD, true) {
                    if choosing {
                        if game.mulligan(&self.mulligan_selection) {
                            self.mulligan_selection.clear();
                        }
                    } else if game.confirm_opening_hands() {
                        self.cpu_continue_at = get_time() + 1.8;
                    }
                }
                draw_centered(
                    if game.player_first {
                        "YOU GO FIRST"
                    } else {
                        "OPPONENT GOES FIRST"
                    },
                    cx - 300.0 * scale,
                    cy + 290.0 * scale,
                    600.0 * scale,
                    19.0 * scale,
                    CYAN,
                );
            }
            SetupStage::StandUp | SetupStage::Complete => {}
        }
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Menu;
        }
    }
}
fn setup_heading(title: &str, detail: &str, scale: f32) {
    draw_centered(
        title,
        0.0,
        screen_height() * 0.17,
        screen_width(),
        36.0 * scale,
        CREAM,
    );
    draw_centered(
        detail,
        20.0,
        screen_height() * 0.17 + 32.0 * scale,
        screen_width() - 40.0,
        17.0 * scale,
        MUTED,
    );
}
fn sign_tile(r: Rect, sign: RpsChoice, accent: Color, selected: bool, enabled: bool) -> bool {
    let hover = enabled && r.contains(mouse_position().into());
    draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(9, 22, 31, 235));
    draw_rectangle_lines(
        r.x,
        r.y,
        r.w,
        r.h,
        if selected || hover { 3.0 } else { 1.0 },
        if selected {
            GOLD
        } else if hover {
            accent
        } else {
            Color::from_rgba(65, 89, 104, 200)
        },
    );
    draw_hand_sign(
        vec2(r.center().x, r.y + r.h * 0.42),
        r.h * 0.24,
        accent,
        sign,
    );
    draw_centered(sign.label(), r.x, r.bottom() - 13.0, r.w, 17.0, CREAM);
    enabled && hover && is_mouse_button_pressed(MouseButton::Left)
}
/// Code-native hand silhouettes; no screenshot artwork is used as a game asset.
fn draw_hand_sign(p: Vec2, size: f32, color: Color, sign: RpsChoice) {
    draw_rectangle(
        p.x - size * 0.55,
        p.y - size * 0.12,
        size * 1.1,
        size * 0.8,
        color,
    );
    let fingers = match sign {
        RpsChoice::Rock => 0,
        RpsChoice::Paper => 4,
        RpsChoice::Scissors => 2,
    };
    for i in 0..4 {
        let x = p.x - size * 0.43 + i as f32 * size * 0.29;
        let y = p.y - size * if i < fingers { 1.0 } else { 0.12 };
        let end = p.y + size * 0.1;
        draw_line(x, y, x, end, size * 0.24, color);
        draw_circle(x, y, size * 0.12, color);
    }
    draw_line(
        p.x - size * 0.55,
        p.y + size * 0.55,
        p.x - size * 0.85,
        p.y + size * 0.16,
        size * 0.28,
        color,
    );
}
pub(super) fn draw_stand_up(progress: f32) {
    let alpha = super::motion::smoothstep(progress / 0.15)
        * (1.0 - super::motion::smoothstep((progress - 0.8) / 0.2));
    let y = screen_height() * 0.45;
    draw_rectangle(
        0.0,
        y - 48.0,
        screen_width(),
        112.0,
        Color::new(0.016, 0.04, 0.067, 0.86 * alpha),
    );
    draw_centered(
        "STAND UP, VANGUARD!",
        0.0,
        y + 29.0,
        screen_width(),
        (screen_width() * 0.045).clamp(28.0, 70.0),
        Color { a: alpha, ..GOLD },
    );
}
pub(super) fn draw_draw_animation(
    art: &TabletopArt,
    game: &MatchGame,
    field: &BattlefieldLayout,
    hand: &HandLayout,
    card_art: &HashMap<String, Texture2D>,
    progress: f32,
) {
    let cpu = game.phase == Phase::Cpu;
    let deck = if cpu {
        &game.cpu.deck
    } else {
        &game.player.deck
    };
    let Some(card) = deck.last() else {
        return;
    };
    let start = field.deck_rect(cpu);
    let hand_slot = hand.draw_destination().filter(|_| !cpu);
    let end = if cpu {
        Rect::new(screen_width() * 0.51, -start.h - 8.0, start.w, start.h)
    } else {
        hand_slot.unwrap_or_else(|| {
            let button = hand.next_page_button();
            let size = vec2(13.0, 19.0);
            let origin = button.center() - size * 0.5;
            Rect::new(origin.x, origin.y, size.x, size.y)
        })
    };
    let travel = super::motion::draw_progress(progress);
    let mut r = super::motion::interpolate_rect(start, end, travel);
    // A restrained lift clears the deck, with no arbitrary spin or oversized flying card.
    r.y -= (std::f32::consts::PI * travel).sin() * 8.0;
    let sleeve = if cpu {
        &art.kai_sleeve
    } else {
        &art.aichi_sleeve
    };
    let flip = if hand_slot.is_none() {
        0.0
    } else {
        ((progress - 0.62) / 0.18).clamp(0.0, 1.0)
    };
    let width = (flip * std::f32::consts::PI).cos().abs().max(0.015);
    let center = r.center().x;
    r.w *= width;
    r.x = center - r.w * 0.5;
    if flip > 0.5 {
        draw_card_face(r, card, false, card_art);
    } else {
        draw_texture_ex(
            sleeve,
            r.x,
            r.y,
            Color {
                a: if !cpu && hand_slot.is_none() {
                    1.0 - super::motion::smoothstep((travel - 0.7) / 0.3)
                } else {
                    1.0
                },
                ..WHITE
            },
            DrawTextureParams {
                dest_size: Some(r.size()),
                rotation: if cpu { std::f32::consts::PI } else { 0.0 },
                ..Default::default()
            },
        );
    }
}
