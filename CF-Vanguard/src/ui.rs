//! Desktop UI. Screens and input stay separate from match rendering and drawing helpers.

mod art;
mod board;
mod cards;
mod drawing;
mod fighters;
mod match_flow;
mod match_hud;
mod match_setup;
mod motion;
mod presentation;

use crate::game::engine::{
    AttackUnit, BattleStep, MatchGame, Phase, RpsChoice, SetupStage, TriggerPart,
};
use crate::story::{SchoolRoute, CHAPTERS};
use art::{normalize_card_name, CardArt};
use board::{draw_battlefield, BattlefieldHighlights, BattlefieldLayout, TabletopArt};
use cards::{draw_guard_card, draw_player_hand, HandLayout};
use drawing::{
    button, draw_ambient_background, draw_centered, draw_header, draw_round_panel,
    draw_school_crest, draw_star_emblem, draw_wrapped, CREAM, CYAN, GOLD, INK, MUTED, PANEL,
    PANEL_LIGHT, RED,
};
use fighters::FighterArt;
use macroquad::prelude::*;
use match_flow::MatchPrompt;
use match_hud::{
    draw_action_panel, draw_battle_log, draw_decision_banner, draw_match_hud, draw_match_shell,
    draw_phase_track, MatchLayout, PowerDisplay,
};
use presentation::PresentationArt;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Screen {
    Title,
    Loading,
    Menu,
    RouteSelect,
    Match,
    Library,
    Result,
}

#[derive(Clone, Copy)]
enum AttackSelection {
    Target {
        attacker: AttackUnit,
        boost: bool,
    },
    Boost {
        attacker: AttackUnit,
        target: AttackUnit,
    },
}

#[derive(Clone, Copy)]
enum GuardSelection {
    Hand(usize),
    Intercept(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MatchAction {
    SkipRide,
    Battle,
    CancelSelection,
    ToggleLog,
    Menu,
    Declare {
        attacker: AttackUnit,
        target: AttackUnit,
        boost: bool,
    },
    Resolve,
    EndTurn,
    LimitBreak,
    Restand,
    Result,
}

pub struct App {
    screen: Screen,
    presentation: PresentationArt,
    loading_stage: u8,
    loading_started: f64,
    route: SchoolRoute,
    story_mode: bool,
    chapter_index: usize,
    match_game: Option<MatchGame>,
    selected_hand_card: Option<usize>,
    hand_page: usize,
    mulligan_selection: Vec<usize>,
    attack_selection: Option<AttackSelection>,
    guard_selection: Option<GuardSelection>,
    guard_press: Option<Vec2>,
    cpu_continue_at: f64,
    tabletop_art: TabletopArt,
    fighters: FighterArt,
    power_display: PowerDisplay,
    card_records: Vec<serde_json::Value>,
    card_art: CardArt,
    show_battle_log: bool,
    library_filter: String,
    library_page: usize,
    selected_library_card: Option<usize>,
}

impl App {
    pub async fn load() -> Self {
        let presentation = PresentationArt::new();
        set_default_font(presentation.font());
        presentation.loading(0.0, "READING CARD DATABASE");
        next_frame().await;
        let database: serde_json::Value =
            serde_json::from_str(include_str!("../assets/official_era_cards.json"))
                .expect("official card metadata must be valid JSON");
        let card_records = database["cards"].as_array().cloned().unwrap_or_default();
        presentation.loading(0.35, "INITIALIZING THE FIGHT FIELD");
        next_frame().await;
        let app = Self {
            screen: Screen::Title,
            presentation,
            loading_stage: 0,
            loading_started: 0.0,
            route: SchoolRoute::Miyaji,
            story_mode: false,
            chapter_index: 0,
            match_game: None,
            selected_hand_card: None,
            hand_page: 0,
            mulligan_selection: Vec::new(),
            attack_selection: None,
            guard_selection: None,
            guard_press: None,
            cpu_continue_at: 0.0,
            tabletop_art: TabletopArt {
                zones: embedded_texture(include_bytes!("../assets/tabletop/vanguard-zones.png")),
                aichi_sleeve: embedded_texture(include_bytes!(
                    "../assets/tabletop/aichi-sleeve.png"
                )),
                kai_sleeve: embedded_texture(include_bytes!("../assets/tabletop/kai-sleeve.png")),
            },
            fighters: FighterArt::new(),
            power_display: PowerDisplay::default(),
            card_art: CardArt::new(&card_records),
            card_records,
            show_battle_log: false,
            library_filter: String::new(),
            library_page: 0,
            selected_library_card: None,
        };
        app.presentation.loading(1.0, "READY TO STAND UP");
        next_frame().await;
        app
    }

    pub fn draw(&mut self) {
        clear_background(INK);
        if matches!(
            self.screen,
            Screen::RouteSelect | Screen::Library | Screen::Result
        ) {
            draw_ambient_background();
        }

        match self.screen {
            Screen::Title => {
                if self.presentation.title() {
                    self.screen = Screen::Menu;
                }
            }
            Screen::Loading => self.draw_match_loading(),
            Screen::Menu => self.draw_menu(),
            Screen::RouteSelect => self.draw_routes(),
            Screen::Match => self.draw_match(),
            Screen::Library => self.draw_library(),
            Screen::Result => self.draw_result(),
        }
    }

    fn draw_menu(&mut self) {
        self.presentation.scene();
        self.presentation.heading();
        if self.presentation.menu_button(
            "STORY MODE",
            "Follow the road from Card Capital to the final fight",
            0,
        ) {
            self.story_mode = true;
            self.screen = Screen::RouteSelect;
        }
        if self.presentation.menu_button(
            "QUICK FIGHT",
            "Stand up your Vanguard and challenge the CPU",
            1,
        ) {
            self.story_mode = false;
            self.chapter_index = 0;
            self.start_match();
        }
        if self.presentation.menu_button(
            "ERA CARD LIBRARY",
            "Explore the Limit Break and Break Ride card pool",
            2,
        ) {
            self.screen = Screen::Library;
        }
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Title;
        }
    }

    fn draw_match_loading(&mut self) {
        let (progress, caption) = match self.loading_stage {
            0 => (0.0, "BUILDING THE FIGHTERS' DECKS"),
            1 => (0.5, "LOADING YOUR OPENING HAND"),
            _ => (1.0, "FIGHT READY"),
        };
        self.presentation.loading(progress, caption);
        if is_key_pressed(KeyCode::Escape) {
            self.match_game = None;
            self.screen = Screen::Menu;
            return;
        }
        match self.loading_stage {
            0 => {
                let mut chapter = CHAPTERS[self.chapter_index];
                if self.chapter_index == 4 {
                    let (rival, clan, boss) = self.route.rival();
                    chapter.rival = rival;
                    chapter.rival_clan = clan;
                    chapter.rival_boss = boss;
                }
                self.match_game = Some(MatchGame::new(chapter));
                self.loading_stage = 1;
            }
            1 => {
                if let Some(game) = &self.match_game {
                    self.card_art.preload(&visible_card_names(game));
                }
                self.loading_stage = 2;
            }
            _ => {
                // A short completed transition gives the opening art time to
                // settle; the progress bar already reports the actual work.
                if get_time() - self.loading_started >= 0.65 {
                    self.screen = Screen::Match;
                }
            }
        }
    }

    fn draw_routes(&mut self) {
        draw_header(
            "CHOOSE YOUR SCHOOL",
            "The Link Joker story has three school paths.",
        );
        let width = screen_width();
        let gap = 22.0;
        let card_w = (width - 100.0 - 2.0 * gap) / 3.0;
        let y = 210.0;
        for (index, route) in SchoolRoute::ALL.into_iter().enumerate() {
            let x = 50.0 + index as f32 * (card_w + gap);
            let selected = self.route == route;
            let accent = if selected { GOLD } else { CYAN };
            draw_round_panel(x, y, card_w, 360.0, PANEL, accent);
            draw_school_crest(x + card_w * 0.5, y + 90.0, accent, index);
            draw_centered(route.name(), x, y + 170.0, card_w, 28.0, CREAM);
            let description = match route {
                SchoolRoute::Hitsue => "Kai's circle at Hitsue High",
                SchoolRoute::Fukuhara => "Ren and the Foo Fighters",
                SchoolRoute::Miyaji => "Aichi's Cardfight Club",
            };
            draw_centered(description, x, y + 205.0, card_w, 17.0, MUTED);
            let (rival, clan, _boss) = route.rival();
            draw_centered("ROUTE RIVAL", x, y + 260.0, card_w, 14.0, GOLD);
            draw_centered(rival, x, y + 289.0, card_w, 18.0, CREAM);
            draw_centered(clan, x, y + 318.0, card_w, 16.0, MUTED);
            if button(
                "SELECT PATH",
                x + 24.0,
                y + 375.0,
                card_w - 48.0,
                50.0,
                accent,
            ) {
                self.route = route;
                self.chapter_index = 0;
                self.start_match();
            }
        }
        if button("BACK", 42.0, 42.0, 120.0, 44.0, MUTED) {
            self.screen = Screen::Menu;
        }
    }

    fn draw_match(&mut self) {
        if self
            .match_game
            .as_ref()
            .is_some_and(|g| g.phase == Phase::Setup)
        {
            if self.match_game.as_ref().unwrap().setup_stage != SetupStage::StandUp {
                self.draw_setup();
                return;
            }
            if get_time() >= self.cpu_continue_at {
                let game = self.match_game.as_mut().unwrap();
                game.begin_fight();
                self.cpu_continue_at = get_time() + phase_delay(game);
            }
        }
        if let Some(game) = self.match_game.as_mut() {
            if matches!(game.phase, Phase::Stand | Phase::Draw)
                && get_time() >= self.cpu_continue_at
            {
                game.advance_player_turn();
                self.cpu_continue_at = get_time() + phase_delay(game);
            }
        }
        if let Some(game) = self.match_game.as_mut() {
            if game.phase == Phase::Cpu
                && game.pending_attack.is_none()
                && get_time() >= self.cpu_continue_at
            {
                game.advance_cpu_turn();
                self.cpu_continue_at = get_time() + phase_delay(game);
            }
        }
        if let Some(game) = self.match_game.as_mut() {
            if game
                .pending_attack
                .as_ref()
                .is_some_and(|a| a.step != BattleStep::Guard && a.decision.is_none())
                && get_time() >= self.cpu_continue_at
            {
                game.resolve_pending_attack();
                self.cpu_continue_at = get_time() + 0.55;
            }
        }
        let card_names = self
            .match_game
            .as_ref()
            .map(visible_card_names)
            .unwrap_or_default();
        self.card_art.preload(&card_names);

        let Some(game) = self.match_game.as_ref() else {
            self.screen = Screen::Menu;
            return;
        };
        let layout = MatchLayout::new(screen_width(), screen_height());
        let battlefield = BattlefieldLayout::new(layout.board);
        let draw_phase = game.stage() == Some(crate::game::engine::TurnStage::Draw);
        let draw_progress =
            (1.0 - (self.cpu_continue_at - get_time()) / 0.8).clamp(0.0, 1.0) as f32;
        let hand = if game.phase == Phase::Draw && !game.player.deck.is_empty() {
            HandLayout::during_draw(
                layout.hand,
                game.player.hand.len(),
                self.hand_page,
                motion::draw_progress(draw_progress),
            )
        } else {
            HandLayout::new(layout.hand, game.player.hand.len(), self.hand_page)
        };
        let mouse = Vec2::from(mouse_position());
        let guardians = game
            .pending_attack
            .as_ref()
            .map_or(&[][..], |attack| attack.guardians.as_slice());

        draw_match_shell(&layout);
        let prompt = MatchPrompt::new(
            game,
            self.attack_selection,
            self.selected_hand_card,
            self.guard_selection,
        );
        draw_battlefield(
            game,
            self.card_art.textures(),
            &battlefield,
            &self.tabletop_art,
            &self.battlefield_highlights(game),
            guardians,
            (game.stage() == Some(crate::game::engine::TurnStage::Stand)).then_some((
                game.phase == Phase::Cpu,
                ((1.5 - (self.cpu_continue_at - get_time()) - 0.35) / 0.7).clamp(0.0, 1.0) as f32,
            )),
        );
        draw_match_hud(
            game,
            &layout,
            self.fighters.portrait("Aichi"),
            self.fighters.portrait(&game.cpu.name),
        );
        draw_phase_track(game, layout.phases);

        draw_decision_banner(&prompt, layout.decision);
        self.power_display.draw(prompt.combat, layout.power);
        let selected_hand = match self.guard_selection {
            Some(GuardSelection::Hand(index)) => Some(index),
            _ => self.selected_hand_card,
        };
        draw_player_hand(game, &hand, selected_hand, self.card_art.textures());
        if game.phase == Phase::Setup {
            match_setup::draw_stand_up(
                (1.0 - (self.cpu_continue_at - get_time()) / 1.8).clamp(0.0, 1.0) as f32,
            );
            return;
        }
        if draw_phase {
            match_setup::draw_draw_animation(
                &self.tabletop_art,
                game,
                &battlefield,
                &hand,
                self.card_art.textures(),
                draw_progress,
            );
        }
        let new_page = hand.draw_page_controls(game.phase != Phase::Draw);
        let action = draw_action_panel(
            game,
            &prompt,
            layout.actions,
            self.show_battle_log,
            self.attack_selection.is_some(),
        );
        if let Some(GuardSelection::Hand(index)) = self.guard_selection {
            if self.is_dragging_guard(mouse)
                && is_mouse_button_down(MouseButton::Left)
                && game.can_guard(index)
            {
                let preview = Rect::new(mouse.x - 40.0, mouse.y - 56.0, 80.0, 54.0);
                draw_guard_card(preview, &game.player.hand[index], self.card_art.textures());
                if battlefield.guardian_at(mouse) {
                    draw_rectangle_lines(preview.x, preview.y, preview.w, preview.h, 2.0, CYAN);
                }
            }
        }

        if self.show_battle_log {
            draw_battle_log(game, layout.log);
        }
        if is_key_pressed(KeyCode::Escape) {
            self.handle_match_clicks(&battlefield, &hand, mouse);
            return;
        }
        if self.show_battle_log && layout.log.contains(mouse) {
            return;
        }
        self.hand_page = hand.page;
        if let Some(page) = new_page {
            self.hand_page = page;
            self.selected_hand_card = None;
            self.guard_selection = None;
            self.guard_press = None;
        } else if let Some(action) = action {
            self.apply_match_action(action);
        } else if !layout.blocks_field_input(mouse) {
            self.handle_match_clicks(&battlefield, &hand, mouse);
        }
    }

    fn battlefield_highlights(&self, game: &MatchGame) -> BattlefieldHighlights {
        let mut highlights = BattlefieldHighlights::default();
        if let Some((part, _)) = game.trigger_decision() {
            if part == TriggerPart::Heal {
                highlights.player_damage = true;
            }
            for target in game.trigger_targets() {
                mark_field_unit(&mut highlights, target, false, GOLD);
            }
            return highlights;
        }
        if let Some(attack) = &game.pending_attack {
            mark_field_unit(&mut highlights, attack.attacker, attack.cpu_attacking, GOLD);
            mark_field_unit(&mut highlights, attack.target, !attack.cpu_attacking, RED);
            if let Some(slot) = attack.booster {
                mark_field_unit(
                    &mut highlights,
                    AttackUnit::Rear(slot),
                    attack.cpu_attacking,
                    CYAN,
                );
            }
            if attack.cpu_attacking && attack.step == BattleStep::Guard {
                highlights.guardian = Some(if self.guard_selection.is_some() {
                    GOLD
                } else {
                    CYAN
                });
                for slot in [1, 3] {
                    if game.can_intercept(slot) {
                        highlights.player_rear[slot] = Some(match self.guard_selection {
                            Some(GuardSelection::Intercept(selected)) if selected == slot => GOLD,
                            _ => CYAN,
                        });
                    }
                }
            } else if !attack.guardians.is_empty() {
                highlights.guardian = Some(GOLD);
            }
            return highlights;
        }
        match (game.phase, self.attack_selection) {
            (Phase::Call, _) if self.selected_hand_card.is_some() => {
                for (slot, unit) in game.player.rear.iter().enumerate() {
                    highlights.player_rear[slot] = Some(if unit.is_none() { CYAN } else { GOLD });
                }
            }
            (Phase::Battle, None) => {
                for attacker in AttackUnit::FRONT_ROW {
                    if game.can_attack(attacker) {
                        mark_field_unit(&mut highlights, attacker, false, CYAN);
                    }
                }
            }
            (Phase::Battle, Some(AttackSelection::Target { attacker, boost })) => {
                mark_field_unit(&mut highlights, attacker, false, GOLD);
                if let Some(slot) = attacker
                    .boost_slot()
                    .filter(|_| game.boost_for(attacker).is_some())
                {
                    highlights.player_rear[slot] = Some(if boost { GOLD } else { CYAN });
                }
                for target in game.attack_targets() {
                    mark_field_unit(&mut highlights, target, true, RED);
                }
            }
            (Phase::Battle, Some(AttackSelection::Boost { attacker, target })) => {
                mark_field_unit(&mut highlights, attacker, false, GOLD);
                mark_field_unit(&mut highlights, target, true, RED);
                if let Some(slot) = attacker.boost_slot() {
                    highlights.player_rear[slot] = Some(CYAN);
                }
            }
            _ => {}
        }
        highlights
    }

    fn apply_match_action(&mut self, action: MatchAction) {
        match action {
            MatchAction::ToggleLog => {
                self.show_battle_log = !self.show_battle_log;
                return;
            }
            MatchAction::Menu => {
                self.screen = Screen::Menu;
                self.guard_press = None;
                return;
            }
            _ => {}
        }
        let Some(game) = self.match_game.as_mut() else {
            return;
        };
        self.selected_hand_card = None;
        self.attack_selection = None;
        self.guard_selection = None;
        self.guard_press = None;
        match action {
            MatchAction::SkipRide => game.skip_ride(),
            MatchAction::Battle => game.begin_battle(),
            MatchAction::CancelSelection => {}
            MatchAction::ToggleLog | MatchAction::Menu => unreachable!(),
            MatchAction::Declare {
                attacker,
                target,
                boost,
            } => {
                game.declare_attack(attacker, target, boost);
            }
            MatchAction::Resolve => {
                game.resolve_pending_attack();
                self.cpu_continue_at = get_time() + 0.65;
            }
            MatchAction::EndTurn => {
                game.end_turn();
                self.cpu_continue_at = get_time() + phase_delay(game);
            }
            MatchAction::LimitBreak => game.use_limit_break(),
            MatchAction::Restand => game.use_restand(),
            MatchAction::Result => self.screen = Screen::Result,
        }
    }

    fn handle_match_clicks(
        &mut self,
        battlefield: &BattlefieldLayout,
        hand: &HandLayout,
        mouse: Vec2,
    ) {
        let Some(game) = self.match_game.as_mut() else {
            return;
        };
        if is_key_pressed(KeyCode::Escape) {
            if self.show_battle_log {
                self.show_battle_log = false;
                return;
            }
            self.guard_press = None;
            let selecting = self.attack_selection.take().is_some()
                | self.guard_selection.take().is_some()
                | self.selected_hand_card.take().is_some();
            if !selecting {
                self.screen = Screen::Menu;
            }
            return;
        }

        if let Some((part, _)) = game.trigger_decision() {
            if is_mouse_button_pressed(MouseButton::Left) {
                let changed = if part == TriggerPart::Heal {
                    battlefield
                        .damage_at(mouse, game.player.damage.len(), false)
                        .is_some_and(|i| game.choose_trigger_damage(i))
                } else {
                    field_unit_at(battlefield, mouse, false)
                        .is_some_and(|unit| game.choose_trigger_unit(unit))
                };
                if changed {
                    self.cpu_continue_at = get_time() + 0.55;
                }
            }
            return;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(index) = battlefield.damage_at(mouse, game.player.damage.len(), false) {
                if game.flip_damage(index) {
                    self.guard_press = None;
                }
                return;
            }
        }

        let playing_guard = game
            .pending_attack
            .as_ref()
            .is_some_and(|attack| attack.cpu_attacking && attack.step == BattleStep::Guard);
        if playing_guard {
            if is_mouse_button_released(MouseButton::Left) {
                let dragged_to_circle = self.guard_press.take().is_some_and(|start| {
                    start.distance(mouse) > 7.0 && battlefield.guardian_at(mouse)
                });
                if dragged_to_circle {
                    if let Some(GuardSelection::Hand(index)) = self.guard_selection.take() {
                        game.call_guardian(index);
                    }
                }
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                if let Some(index) = hand.card_at(mouse).filter(|&index| game.can_guard(index)) {
                    self.guard_selection = Some(GuardSelection::Hand(index));
                    self.guard_press = Some(mouse);
                } else if let Some(slot) = battlefield
                    .rear_at(mouse, false)
                    .filter(|&slot| game.can_intercept(slot))
                {
                    self.guard_selection = Some(GuardSelection::Intercept(slot));
                    self.guard_press = None;
                } else if battlefield.guardian_at(mouse) {
                    match self.guard_selection.take() {
                        Some(GuardSelection::Hand(index)) => {
                            game.call_guardian(index);
                        }
                        Some(GuardSelection::Intercept(slot)) => {
                            game.intercept(slot);
                        }
                        None => {}
                    }
                    self.guard_press = None;
                }
            }
            return;
        }
        if game.pending_attack.is_some() || !is_mouse_button_pressed(MouseButton::Left) {
            return;
        }

        match game.phase {
            Phase::Ride => {
                if let Some(index) = hand
                    .card_at(mouse)
                    .filter(|index| game.ride_options().contains(index))
                {
                    game.ride(index);
                }
            }
            Phase::Call => {
                if let Some(index) = hand.card_at(mouse) {
                    self.selected_hand_card = game.call_options().contains(&index).then_some(index);
                } else if let Some(slot) = battlefield.rear_at(mouse, false) {
                    if let Some(index) = self.selected_hand_card.take() {
                        game.call(index, slot);
                    }
                }
            }
            Phase::Battle => {
                let friendly = field_unit_at(battlefield, mouse, false);
                let enemy = field_unit_at(battlefield, mouse, true);
                match self.attack_selection {
                    None => {
                        if let Some(attacker) = friendly.filter(|&unit| game.can_attack(unit)) {
                            self.attack_selection = Some(AttackSelection::Target {
                                attacker,
                                boost: false,
                            });
                        } else if let Some(attacker) = AttackUnit::FRONT_ROW.into_iter().find(|a| {
                            game.can_attack(*a)
                                && game.boost_for(*a).is_some()
                                && friendly == a.boost_slot().map(AttackUnit::Rear)
                        }) {
                            self.attack_selection = Some(AttackSelection::Target {
                                attacker,
                                boost: true,
                            });
                        }
                    }
                    Some(AttackSelection::Target { attacker, boost }) => {
                        if let Some(target) =
                            enemy.filter(|target| game.attack_targets().contains(target))
                        {
                            if boost {
                                game.declare_attack(attacker, target, true);
                                self.attack_selection = None;
                            } else if game.boost_for(attacker).is_some() {
                                self.attack_selection =
                                    Some(AttackSelection::Boost { attacker, target });
                            } else {
                                game.declare_attack(attacker, target, false);
                                self.attack_selection = None;
                            }
                        } else if game.boost_for(attacker).is_some()
                            && friendly == attacker.boost_slot().map(AttackUnit::Rear)
                        {
                            self.attack_selection = Some(AttackSelection::Target {
                                attacker,
                                boost: !boost,
                            });
                        } else if let Some(next) = friendly.filter(|&unit| game.can_attack(unit)) {
                            self.attack_selection =
                                (next != attacker).then_some(AttackSelection::Target {
                                    attacker: next,
                                    boost: false,
                                });
                        }
                    }
                    Some(AttackSelection::Boost { attacker, target }) => {
                        if battlefield
                            .rear_at(mouse, false)
                            .is_some_and(|slot| Some(slot) == attacker.boost_slot())
                        {
                            game.declare_attack(attacker, target, true);
                            self.attack_selection = None;
                        }
                    }
                }
            }
            Phase::Setup | Phase::Stand | Phase::Draw | Phase::Cpu | Phase::Finished => {}
        }
    }

    fn is_dragging_guard(&self, mouse: Vec2) -> bool {
        self.guard_press
            .is_some_and(|start| start.distance(mouse) > 7.0)
    }

    fn draw_result(&mut self) {
        let Some(game) = self.match_game.as_ref() else {
            self.screen = Screen::Menu;
            return;
        };
        draw_header(
            if game.winner == Some(true) {
                "CARD FIGHT WON"
            } else {
                "CARD FIGHT LOST"
            },
            &game.chapter.title,
        );
        let center = screen_width() * 0.5;
        draw_star_emblem(center, 325.0, 75.0);
        draw_centered(
            &game.message,
            50.0,
            440.0,
            screen_width() - 100.0,
            25.0,
            CREAM,
        );
        draw_centered(
            &format!(
                "{} damage  ·  {} damage",
                game.player.damage.len(),
                game.cpu.damage.len()
            ),
            50.0,
            488.0,
            screen_width() - 100.0,
            18.0,
            MUTED,
        );
        draw_centered(
            game.chapter.synopsis,
            90.0,
            116.0,
            screen_width() - 180.0,
            16.0,
            MUTED,
        );
        if game.winner == Some(true) {
            draw_centered(
                game.chapter.reward,
                50.0,
                520.0,
                screen_width() - 100.0,
                16.0,
                GOLD,
            );
        }

        if game.winner == Some(true) && self.story_mode && self.chapter_index + 1 < CHAPTERS.len() {
            if button("NEXT CHAPTER", center - 140.0, 560.0, 280.0, 64.0, GOLD) {
                self.chapter_index += 1;
                self.start_match();
            }
        } else if button("RETURN TO MENU", center - 140.0, 560.0, 280.0, 64.0, CYAN) {
            self.screen = Screen::Menu;
        }
    }

    fn draw_library(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            if self.selected_library_card.take().is_none() {
                self.screen = Screen::Menu;
            }
        }

        if is_key_pressed(KeyCode::Backspace) {
            self.library_filter.pop();
            self.library_page = 0;
        }
        while let Some(character) = get_char_pressed() {
            if !character.is_control() {
                self.library_filter.push(character);
                self.library_page = 0;
            }
        }

        let query = self.library_filter.to_lowercase();
        let filtered: Vec<usize> = self
            .card_records
            .iter()
            .enumerate()
            .filter_map(|(index, record)| {
                let matches = query.is_empty()
                    || ["card_name", "card_number", "clan", "product"]
                        .iter()
                        .filter_map(|field| record[*field].as_str())
                        .any(|value| value.to_lowercase().contains(&query));
                matches.then_some(index)
            })
            .collect();
        let cards_per_page = 12;
        let page_count = filtered.len().div_ceil(cards_per_page).max(1);
        self.library_page = self.library_page.min(page_count - 1);
        let first = self.library_page * cards_per_page;
        let visible_indices = filtered
            .iter()
            .skip(first)
            .take(cards_per_page)
            .copied()
            .collect::<Vec<_>>();
        let visible_cards = visible_indices
            .iter()
            .map(|index| self.card_records[*index].clone())
            .collect::<Vec<_>>();
        let visible_names = visible_cards
            .iter()
            .filter_map(|card| card["card_name"].as_str().map(str::to_owned))
            .collect::<Vec<_>>();
        self.card_art.preload(&visible_names);

        draw_header(
            "3DS-ERA CARD ARCHIVE",
            "Official card entries for BT01–BT15, EB01–EB10, and TD01–TD13.",
        );
        if button("BACK", 42.0, 42.0, 120.0, 44.0, MUTED) {
            self.screen = Screen::Menu;
        }

        draw_round_panel(190.0, 113.0, screen_width() - 230.0, 42.0, PANEL, CYAN);
        draw_text("SEARCH", 207.0, 139.0, 13.0, GOLD);
        draw_text(
            if self.library_filter.is_empty() {
                "Type a card name, number, clan, or set"
            } else {
                &self.library_filter
            },
            282.0,
            140.0,
            16.0,
            CREAM,
        );
        draw_text(
            &format!("{} matches", filtered.len()),
            screen_width() - 160.0,
            140.0,
            13.0,
            MUTED,
        );

        for (slot, (index, record)) in visible_indices.iter().zip(&visible_cards).enumerate() {
            let col = slot % 4;
            let row = slot / 4;
            let x = 42.0 + col as f32 * 348.0;
            let y = 170.0 + row as f32 * 205.0;
            let item = Rect::new(x, y, 330.0, 188.0);
            draw_round_panel(x, y, item.w, item.h, PANEL, PANEL_LIGHT);
            let name = record["card_name"].as_str().unwrap_or("Unknown card");
            let key = normalize_card_name(name);
            if let Some(texture) = self.card_art.textures().get(&key) {
                draw_texture_ex(
                    texture,
                    x + 10.0,
                    y + 9.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(106.0, 154.0)),
                        ..Default::default()
                    },
                );
            } else {
                draw_round_panel(x + 10.0, y + 9.0, 106.0, 154.0, PANEL_LIGHT, GOLD);
                draw_centered("NO IMAGE", x + 10.0, y + 87.0, 106.0, 12.0, MUTED);
            }

            let text_x = x + 128.0;
            draw_wrapped(name, text_x, y + 28.0, 190.0, 13.0, CREAM, 3);
            draw_text(
                record["card_number"].as_str().unwrap_or(""),
                text_x,
                y + 84.0,
                12.0,
                GOLD,
            );
            draw_text(
                &format!(
                    "{}  ·  G{}  ·  {}",
                    record["clan"].as_str().unwrap_or("Clan"),
                    record["grade"].as_str().unwrap_or("?"),
                    record["power"].as_str().unwrap_or("—")
                ),
                text_x,
                y + 108.0,
                11.0,
                MUTED,
            );
            draw_wrapped(
                record["product"].as_str().unwrap_or(""),
                text_x,
                y + 135.0,
                190.0,
                10.0,
                MUTED,
                2,
            );

            let mouse = mouse_position();
            if item.contains(vec2(mouse.0, mouse.1)) && is_mouse_button_pressed(MouseButton::Left) {
                self.selected_library_card = Some(*index);
            }
        }

        draw_text(
            &format!("PAGE {} / {}", self.library_page + 1, page_count),
            screen_width() * 0.5 - 60.0,
            screen_height() - 24.0,
            13.0,
            MUTED,
        );
        if self.library_page > 0
            && button(
                "PREVIOUS",
                screen_width() * 0.5 - 170.0,
                screen_height() - 52.0,
                110.0,
                36.0,
                CYAN,
            )
        {
            self.library_page -= 1;
        }
        if self.library_page + 1 < page_count
            && button(
                "NEXT",
                screen_width() * 0.5 + 60.0,
                screen_height() - 52.0,
                110.0,
                36.0,
                CYAN,
            )
        {
            self.library_page += 1;
        }

        if let Some(index) = self.selected_library_card {
            if let Some(record) = self.card_records.get(index) {
                draw_card_detail(record, self.card_art.textures());
            }
            if button("CLOSE", screen_width() - 160.0, 30.0, 110.0, 42.0, GOLD) {
                self.selected_library_card = None;
            }
        }
    }

    fn start_match(&mut self) {
        self.power_display = PowerDisplay::default();
        self.mulligan_selection.clear();
        self.match_game = None;
        self.loading_stage = 0;
        self.loading_started = get_time();
        self.selected_hand_card = None;
        self.hand_page = 0;
        self.attack_selection = None;
        self.guard_selection = None;
        self.guard_press = None;
        self.cpu_continue_at = 0.0;
        self.show_battle_log = false;
        self.screen = Screen::Loading;
    }
}

fn embedded_texture(bytes: &[u8]) -> Texture2D {
    let texture = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
    texture.set_filter(FilterMode::Linear);
    texture
}

fn field_unit_at(layout: &BattlefieldLayout, position: Vec2, cpu: bool) -> Option<AttackUnit> {
    if layout.vanguard_at(position, cpu) {
        Some(AttackUnit::Vanguard)
    } else {
        layout.rear_at(position, cpu).map(AttackUnit::Rear)
    }
}

fn mark_field_unit(
    highlights: &mut BattlefieldHighlights,
    unit: AttackUnit,
    cpu: bool,
    color: Color,
) {
    match (cpu, unit) {
        (false, AttackUnit::Vanguard) => highlights.player_vanguard = Some(color),
        (true, AttackUnit::Vanguard) => highlights.cpu_vanguard = Some(color),
        (false, AttackUnit::Rear(slot)) => highlights.player_rear[slot] = Some(color),
        (true, AttackUnit::Rear(slot)) => highlights.cpu_rear[slot] = Some(color),
    }
}

fn unit_name(unit: AttackUnit, player: &crate::game::card::Player) -> &'static str {
    unit.unit(player).map_or("Unit", |unit| unit.card.name)
}

fn visible_card_names(game: &MatchGame) -> Vec<String> {
    let mut names: Vec<_> = game
        .player
        .hand
        .iter()
        .map(|card| card.name.to_owned())
        .collect();
    if let Some(attack) = &game.pending_attack {
        names.extend(attack.guardians.iter().map(|card| card.name.to_string()));
    }
    for player in [&game.player, &game.cpu] {
        if let Some(card) = player.drop_zone.last() {
            names.push(card.name.to_owned());
        }
        if let Some(check) = &player.last_check {
            names.push(checked_card_name(check).to_owned());
        }
        if let Some(vanguard) = &player.vanguard {
            names.push(vanguard.card.name.to_string());
        }
        names.extend(
            player
                .damage
                .iter()
                .filter(|card| card.face_up)
                .map(|card| card.card.name.to_owned()),
        );
        names.extend(
            player
                .rear
                .iter()
                .flatten()
                .map(|unit| unit.card.name.to_string()),
        );
    }
    if game.phase == Phase::Draw {
        if let Some(card) = game.player.deck.last() {
            names.push(card.name.to_owned());
        }
    }
    names
}

fn checked_card_name(check: &str) -> &str {
    check
        .strip_prefix("Drive: ")
        .or_else(|| check.strip_prefix("Damage: "))
        .unwrap_or(check)
}

fn draw_card_detail(card: &serde_json::Value, textures: &HashMap<String, Texture2D>) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.005, 0.01, 0.025, 0.88),
    );
    let panel_x = screen_width() * 0.5 - 430.0;
    let panel_y = 150.0;
    draw_round_panel(panel_x, panel_y, 860.0, 640.0, PANEL, GOLD);

    let name = card["card_name"].as_str().unwrap_or("Unknown card");
    let image_key = normalize_card_name(name);
    if let Some(texture) = textures.get(&image_key) {
        draw_texture_ex(
            texture,
            panel_x + 34.0,
            panel_y + 45.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(250.0, 365.0)),
                ..Default::default()
            },
        );
    }
    let text_x = panel_x + 320.0;
    draw_wrapped(name, text_x, panel_y + 64.0, 500.0, 23.0, CREAM, 2);
    draw_text(
        card["card_number"].as_str().unwrap_or(""),
        text_x,
        panel_y + 126.0,
        15.0,
        GOLD,
    );
    draw_text(
        &format!(
            "{} · G{} · {} power · {} shield",
            card["clan"].as_str().unwrap_or("Clan"),
            card["grade"].as_str().unwrap_or("?"),
            card["power"].as_str().unwrap_or("?"),
            card["shield"].as_str().unwrap_or("?")
        ),
        text_x,
        panel_y + 157.0,
        14.0,
        CYAN,
    );

    let rules_text = card["text"]
        .as_str()
        .unwrap_or("No ability text is listed.")
        .replace("<br>", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">");
    draw_wrapped(&rules_text, text_x, panel_y + 200.0, 492.0, 13.0, CREAM, 19);
    draw_text(
        card["product"].as_str().unwrap_or(""),
        text_x,
        panel_y + 600.0,
        12.0,
        MUTED,
    );
}

/// Keeps automatic Stand and Draw readable before the next decision phase.
fn phase_delay(game: &MatchGame) -> f64 {
    match game.stage() {
        Some(crate::game::engine::TurnStage::Stand) => 1.5,
        Some(crate::game::engine::TurnStage::Draw) => 0.8,
        _ => 0.65,
    }
}
