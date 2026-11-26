//! Card-first deck workshop with shared logical drawing and pointer coordinates.

use super::art::CardArt;
use super::drawing::{
    draw_centered, draw_round_panel, draw_wrapped, CREAM, CYAN, GOLD, INK, MUTED, PANEL,
    PANEL_LIGHT, RED,
};
use crate::game::card::Card;
use crate::game::deck_editor::{
    field, is_trigger, nation, number, CardSort, CatalogFilter, EditedDeck, NATIONS,
};
use macroquad::prelude::*;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

const WIDTH: f32 = 1440.0;
const HEIGHT: f32 = 960.0;
const GREEN: Color = Color::new(0.40, 0.87, 0.67, 1.0);

pub(super) enum WorkshopAction {
    None,
    Menu,
    Fight,
}

pub(super) struct DeckWorkshop {
    logo: Texture2D,
    sleeve: Texture2D,
    icons: HashMap<&'static str, Texture2D>,
    filter: CatalogFilter,
    deck: EditedDeck,
    saved: Option<EditedDeck>,
    page: usize,
    deck_page: usize,
    selected: Option<usize>,
    searching: bool,
    sort_open: bool,
    details: bool,
    status: String,
}

impl DeckWorkshop {
    pub(super) fn new(records: &[Value]) -> Self {
        let (saved, status) = match EditedDeck::load(records) {
            Ok(Some(deck)) => (Some(deck), "Saved Quick Fight deck loaded.".into()),
            Ok(None) => (
                None,
                "Choose a clan and load a sample, or build your own deck.".into(),
            ),
            Err(error) => (None, format!("Could not load saved deck: {error}")),
        };
        let deck = EditedDeck {
            cards: saved.as_ref().map(|d| d.cards.clone()).unwrap_or_default(),
        };
        Self {
            logo: super::embedded_texture(include_bytes!(
                "../../assets/presentation/vanguard-logo.png"
            )),
            sleeve: super::embedded_texture(include_bytes!("../../assets/tabletop/kai-sleeve.png")),
            icons: clan_icons(),
            filter: CatalogFilter::default(),
            deck,
            saved,
            page: 0,
            deck_page: 0,
            selected: None,
            searching: false,
            sort_open: false,
            details: false,
            status,
        }
    }

    pub(super) fn quick_fight_deck(&self, records: &[Value]) -> Option<Vec<Card>> {
        self.saved.as_ref()?.playable(records).ok()
    }

    pub(super) fn draw(&mut self, records: &[Value], art: &mut CardArt) -> WorkshopAction {
        // Use the entire viewport, with the same transform for drawing and input.
        let mouse = vec2(
            mouse_position().0 * WIDTH / screen_width(),
            mouse_position().1 * HEIGHT / screen_height(),
        );
        let camera = Camera2D {
            target: vec2(WIDTH * 0.5, HEIGHT * 0.5),
            zoom: vec2(2.0 / WIDTH, 2.0 / HEIGHT),
            ..Default::default()
        };
        set_camera(&camera);
        let action = self.draw_content(records, art, mouse);
        set_default_camera();
        action
    }

    fn draw_content(
        &mut self,
        records: &[Value],
        art: &mut CardArt,
        mouse: Vec2,
    ) -> WorkshopAction {
        if is_key_pressed(KeyCode::Escape) {
            if self.details {
                self.details = false;
            } else if self.sort_open {
                self.sort_open = false;
            } else if self.searching {
                self.searching = false;
            } else {
                return WorkshopAction::Menu;
            }
        }
        if self.searching && !self.details {
            let old = self.filter.query.clone();
            if is_key_pressed(KeyCode::Backspace) {
                self.filter.query.pop();
            }
            while let Some(c) = get_char_pressed() {
                if !c.is_control() && self.filter.query.chars().count() < 64 {
                    self.filter.query.push(c);
                }
            }
            if is_key_pressed(KeyCode::Enter) {
                self.searching = false;
            }
            if old != self.filter.query {
                self.page = 0;
            }
        } else {
            while get_char_pressed().is_some() {}
        }
        let interactive = !self.details && !self.sort_open;
        draw_rectangle(0.0, 0.0, WIDTH, HEIGHT, INK);
        for i in 0..16 {
            draw_line(
                240.0 + i as f32 * 85.0,
                94.0,
                100.0 + i as f32 * 85.0,
                HEIGHT,
                1.0,
                Color::new(0.17, 0.35, 0.48, 0.045),
            );
        }
        draw_rectangle(0.0, 0.0, WIDTH, 94.0, Color::new(0.025, 0.04, 0.075, 1.0));
        draw_texture_ex(
            &self.logo,
            28.0,
            12.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(204.0, 204.0 * self.logo.height() / self.logo.width())),
                ..Default::default()
            },
        );
        draw_text("DECK BUILD", 264.0, 49.0, 32.0, CREAM);
        draw_text("LIMIT BREAK / BREAK RIDE", 265.0, 74.0, 13.0, MUTED);
        draw_text("COLLECTION  /  MAIN DECK", 609.0, 53.0, 17.0, CYAN);
        draw_text(
            "BT01 - BT15   /   EB01 - EB10   /   TD01 - TD13",
            900.0,
            53.0,
            13.0,
            MUTED,
        );
        if control(
            "BACK TO MENU",
            Rect::new(1250.0, 28.0, 166.0, 40.0),
            false,
            MUTED,
            mouse,
            interactive,
        ) {
            return WorkshopAction::Menu;
        }
        self.draw_filters(records, mouse, interactive);
        self.draw_toolbar(mouse, interactive);

        let filtered = self.filter.results(records);
        let rows = catalog_rows(&filtered, records, self.filter.sort);
        let pages = rows.len().div_ceil(2).max(1);
        self.page = self.page.min(pages - 1);
        let visible = rows
            .iter()
            .skip(self.page * 2)
            .take(2)
            .flat_map(|row| row.cards.iter().copied())
            .collect::<Vec<_>>();
        if self.selected.is_none() {
            self.selected = visible.first().copied();
        }
        let indices = visible
            .iter()
            .chain(self.deck.cards.iter())
            .chain(self.selected.iter())
            .copied()
            .collect::<Vec<_>>();
        art.preload_records(&indices, records);
        draw_text(
            &format!("COLLECTION   /   {} CARDS", filtered.len()),
            254.0,
            207.0,
            14.0,
            MUTED,
        );
        if control(
            "<",
            Rect::new(895.0, 186.0, 34.0, 28.0),
            false,
            CYAN,
            mouse,
            interactive && self.page > 0,
        ) {
            self.page -= 1;
        }
        draw_centered(
            &format!("{} / {}", self.page + 1, pages),
            932.0,
            206.0,
            92.0,
            14.0,
            CREAM,
        );
        if control(
            ">",
            Rect::new(1032.0, 186.0, 34.0, 28.0),
            false,
            CYAN,
            mouse,
            interactive && self.page + 1 < pages,
        ) {
            self.page += 1;
        }
        if interactive && Rect::new(244.0, 220.0, 830.0, 475.0).contains(mouse) {
            let wheel = mouse_wheel().1;
            if wheel < 0.0 || is_key_pressed(KeyCode::Right) {
                self.page = (self.page + 1).min(pages - 1);
            }
            if wheel > 0.0 || is_key_pressed(KeyCode::Left) {
                self.page = self.page.saturating_sub(1);
            }
        }
        if filtered.is_empty() {
            draw_round_panel(250.0, 235.0, 818.0, 440.0, PANEL, PANEL_LIGHT);
            draw_centered(
                "NO CARDS MATCH THESE FILTERS",
                250.0,
                419.0,
                818.0,
                24.0,
                CREAM,
            );
            draw_centered(
                "Try another grade, clan, or search.",
                250.0,
                451.0,
                818.0,
                17.0,
                MUTED,
            );
            if control(
                "RESET FILTERS",
                Rect::new(545.0, 477.0, 220.0, 42.0),
                false,
                CYAN,
                mouse,
                interactive,
            ) {
                self.filter = CatalogFilter::default();
                self.page = 0;
            }
        }
        for (row_index, row) in rows.iter().skip(self.page * 2).take(2).enumerate() {
            let y = 230.0 + row_index as f32 * 232.0;
            let accent = nation_color(nation(&records[row.cards[0]]));
            draw_rectangle(252.0, y, 3.0, 16.0, accent);
            draw_text(&row.label, 264.0, y + 13.0, 13.0, accent);
            draw_line(252.0, y + 23.0, 1065.0, y + 23.0, 1.0, PANEL_LIGHT);
            for (column, i) in row.cards.iter().enumerate() {
                let rect = Rect::new(252.0 + column as f32 * 165.0, y + 31.0, 153.0, 193.0);
                let selected = self.selected == Some(*i);
                let hover = rect.contains(mouse) && interactive;
                draw_round_panel(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    if hover { PANEL_LIGHT } else { PANEL },
                    if selected { CYAN } else { PANEL_LIGHT },
                );
                thumbnail(
                    &records[*i],
                    art,
                    Rect::new(rect.x + 23.0, rect.y + 7.0, 106.0, 154.0),
                );
                let copies = self.deck.copies(*i, records);
                if copies > 0 {
                    badge(&format!("x{copies}"), rect.x + 111.0, rect.y + 9.0, CYAN);
                }
                badge(
                    &format!("G{}", field(&records[*i], "grade")),
                    rect.x + 7.0,
                    rect.y + 9.0,
                    accent,
                );
                clipped_text(
                    field(&records[*i], "card_name"),
                    rect.x + 8.0,
                    rect.y + 178.0,
                    139.0,
                    13.0,
                    CREAM,
                );
                if hover {
                    if is_mouse_button_pressed(MouseButton::Left) {
                        self.selected = Some(*i);
                        self.searching = false;
                    }
                    if is_mouse_button_pressed(MouseButton::Right) {
                        self.deck.remove(*i);
                        self.status = "Removed a copy from your deck.".into();
                    }
                }
            }
        }
        self.draw_deck(records, art, mouse, interactive);
        self.draw_preview(records, art, mouse, interactive);
        let mut action = WorkshopAction::None;
        if self.draw_statistics(records, mouse, interactive) {
            action = WorkshopAction::Fight;
        }
        draw_line(24.0, 912.0, 1416.0, 912.0, 1.0, PANEL_LIGHT);
        clipped_text(&self.status, 26.0, 938.0, 890.0, 15.0, MUTED);
        draw_text(
            "CLICK: INSPECT    RIGHT CLICK: REMOVE    SCROLL: BROWSE",
            969.0,
            938.0,
            12.0,
            MUTED,
        );
        if self.sort_open {
            self.draw_sort_menu(mouse);
        }
        if self.details {
            if let Some(i) = self.selected {
                draw_details(&records[i], art);
            }
            if control(
                "CLOSE",
                Rect::new(1210.0, 112.0, 100.0, 38.0),
                false,
                CYAN,
                mouse,
                true,
            ) {
                self.details = false;
            }
        }
        action
    }

    fn draw_filters(&mut self, records: &[Value], mouse: Vec2, enabled: bool) {
        draw_round_panel(24.0, 114.0, 208.0, 782.0, PANEL, PANEL_LIGHT);
        draw_text("BROWSE BY NATION", 38.0, 140.0, 14.0, MUTED);
        if control(
            "ALL CARDS",
            Rect::new(36.0, 153.0, 184.0, 33.0),
            self.filter.nation.is_none() && self.filter.clan.is_none(),
            CYAN,
            mouse,
            enabled,
        ) {
            let sort = self.filter.sort;
            self.filter = CatalogFilter {
                sort,
                ..Default::default()
            };
            self.page = 0;
        }
        let mut y = 196.0;
        for country in NATIONS {
            let accent = nation_color(country);
            let rect = Rect::new(36.0, y, 184.0, 27.0);
            draw_rectangle(rect.x, rect.y + 5.0, 3.0, 17.0, accent);
            if control(
                country,
                Rect::new(43.0, y, 177.0, 27.0),
                self.filter.nation == Some(country) && self.filter.clan.is_none(),
                accent,
                mouse,
                enabled,
            ) {
                self.filter.nation = Some(country);
                self.filter.clan = None;
                self.page = 0;
            }
            y += 30.0;
            let mut clans = BTreeMap::new();
            for card in records.iter().filter(|r| nation(r) == country) {
                *clans.entry(field(card, "clan")).or_insert(0) += 1;
            }
            for (clan, count) in clans {
                let rect = Rect::new(47.0, y, 169.0, 21.0);
                let active = self.filter.clan.as_deref() == Some(clan);
                if active || rect.contains(mouse) && enabled {
                    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL_LIGHT);
                }
                if active {
                    draw_rectangle(rect.x, rect.y, 2.0, rect.h, accent);
                }
                if let Some(icon) = self.icons.get(clan) {
                    if country == "Magallanica" {
                        draw_circle(
                            rect.x + 14.5,
                            y + 10.5,
                            10.0,
                            Color::new(0.75, 0.82, 0.94, 1.0),
                        );
                    }
                    draw_texture_ex(
                        icon,
                        rect.x + 5.0,
                        y + 1.0,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(19.0, 19.0)),
                            ..Default::default()
                        },
                    );
                }
                clipped_text(
                    clan,
                    rect.x + 30.0,
                    y + 15.0,
                    111.0,
                    13.0,
                    if active { CREAM } else { MUTED },
                );
                draw_text(&format!("{count}"), 192.0, y + 14.0, 11.0, accent);
                if enabled && rect.contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
                    self.filter.nation = Some(country);
                    self.filter.clan = Some(clan.into());
                    self.page = 0;
                }
                y += 21.0;
            }
            y += 6.0;
        }
    }

    fn draw_toolbar(&mut self, mouse: Vec2, enabled: bool) {
        let search = Rect::new(252.0, 114.0, 546.0, 37.0);
        draw_round_panel(
            search.x,
            search.y,
            search.w,
            search.h,
            PANEL,
            if self.searching { CYAN } else { PANEL_LIGHT },
        );
        draw_text("SEARCH", 265.0, 139.0, 12.0, CYAN);
        clipped_text(
            if self.filter.query.is_empty() {
                "Card name, clan, set, or ability..."
            } else {
                &self.filter.query
            },
            325.0,
            139.0,
            431.0,
            16.0,
            if self.filter.query.is_empty() {
                MUTED
            } else {
                CREAM
            },
        );
        if self.searching && (get_time() * 2.0) as i32 % 2 == 0 {
            draw_rectangle(780.0, 125.0, 2.0, 17.0, CYAN);
        }
        if enabled && is_mouse_button_pressed(MouseButton::Left) {
            self.searching = search.contains(mouse);
        }
        if control(
            "x",
            Rect::new(760.0, 120.0, 31.0, 25.0),
            false,
            MUTED,
            mouse,
            enabled,
        ) {
            self.filter.query.clear();
            self.page = 0;
        }
        let sort_label = format!("SORT: {}  v", self.filter.sort.label());
        if control(
            &sort_label,
            Rect::new(811.0, 114.0, 256.0, 37.0),
            false,
            CYAN,
            mouse,
            enabled,
        ) {
            self.sort_open = true;
        }
        draw_text("Grade:", 253.0, 177.0, 17.0, CREAM);
        if control(
            "All",
            Rect::new(310.0, 157.0, 48.0, 27.0),
            self.filter.grades == 0,
            CYAN,
            mouse,
            enabled,
        ) {
            self.filter.grades = 0;
            self.page = 0;
        }
        for grade in 0..=4 {
            if control(
                &grade.to_string(),
                Rect::new(365.0 + grade as f32 * 39.0, 157.0, 32.0, 27.0),
                self.filter.grades & (1 << grade) != 0,
                CYAN,
                mouse,
                enabled,
            ) {
                self.filter.grades ^= 1 << grade;
                self.page = 0;
            }
        }
        draw_text("Select multiple grades", 576.0, 176.0, 12.0, MUTED);
        if control(
            "RESET",
            Rect::new(994.0, 157.0, 73.0, 27.0),
            false,
            MUTED,
            mouse,
            enabled,
        ) {
            self.filter = CatalogFilter::default();
            self.page = 0;
        }
    }

    fn draw_sort_menu(&mut self, mouse: Vec2) {
        draw_rectangle(
            0.0,
            94.0,
            WIDTH,
            HEIGHT - 94.0,
            Color::new(0.0, 0.0, 0.0, 0.25),
        );
        draw_round_panel(811.0, 152.0, 256.0, 183.0, PANEL, CYAN);
        for (i, sort) in CardSort::ALL.iter().enumerate() {
            if control(
                sort.label(),
                Rect::new(819.0, 159.0 + i as f32 * 34.0, 240.0, 30.0),
                self.filter.sort == *sort,
                CYAN,
                mouse,
                true,
            ) {
                self.filter.sort = *sort;
                self.page = 0;
                self.sort_open = false;
            }
        }
        if is_mouse_button_pressed(MouseButton::Left)
            && !Rect::new(811.0, 114.0, 256.0, 221.0).contains(mouse)
        {
            self.sort_open = false;
        }
    }

    fn draw_deck(&mut self, records: &[Value], art: &CardArt, mouse: Vec2, enabled: bool) {
        draw_round_panel(250.0, 704.0, 818.0, 192.0, PANEL, PANEL_LIGHT);
        draw_text("YOUR DECK", 265.0, 729.0, 17.0, CREAM);
        draw_text(
            &format!("{} / 50", self.deck.cards.len()),
            370.0,
            729.0,
            17.0,
            CYAN,
        );
        if control(
            "NEW",
            Rect::new(675.0, 712.0, 62.0, 26.0),
            false,
            MUTED,
            mouse,
            enabled,
        ) {
            self.deck.cards.clear();
            self.status = "New draft started. Your saved Quick Fight deck is unchanged.".into();
        }
        if control(
            "LOAD CLAN SAMPLE",
            Rect::new(747.0, 712.0, 154.0, 26.0),
            false,
            CYAN,
            mouse,
            enabled && self.filter.clan.is_some(),
        ) {
            if let Some(clan) = self.filter.clan.as_ref() {
                self.deck = EditedDeck::sample(clan, records);
                self.deck_page = 0;
                self.status = format!("{clan} sample loaded. Edit it, then save for Quick Fight.");
            }
        }
        let mut unique = BTreeMap::new();
        for i in &self.deck.cards {
            *unique.entry(*i).or_insert(0usize) += 1;
        }
        let mut entries = unique.into_iter().collect::<Vec<_>>();
        entries.sort_by_key(|(i, _)| {
            (
                number(&records[*i], "grade"),
                field(&records[*i], "card_name"),
            )
        });
        let pages = entries.len().div_ceil(12).max(1);
        self.deck_page = self.deck_page.min(pages - 1);
        if control(
            "<",
            Rect::new(920.0, 712.0, 26.0, 26.0),
            false,
            CYAN,
            mouse,
            enabled && self.deck_page > 0,
        ) {
            self.deck_page -= 1;
        }
        draw_centered(
            &format!("{}/{}", self.deck_page + 1, pages),
            946.0,
            730.0,
            72.0,
            12.0,
            MUTED,
        );
        if control(
            ">",
            Rect::new(1020.0, 712.0, 26.0, 26.0),
            false,
            CYAN,
            mouse,
            enabled && self.deck_page + 1 < pages,
        ) {
            self.deck_page += 1;
        }
        if self.deck.cards.is_empty() {
            draw_texture_ex(
                &self.sleeve,
                276.0,
                748.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(74.0, 108.0)),
                    ..Default::default()
                },
            );
            draw_centered("Stand up your Vanguard.", 365.0, 799.0, 670.0, 22.0, CREAM);
            draw_centered(
                "Choose a clan and load a sample, or select a card to begin.",
                365.0,
                831.0,
                670.0,
                16.0,
                MUTED,
            );
        }
        for (slot, (i, count)) in entries
            .iter()
            .skip(self.deck_page * 12)
            .take(12)
            .enumerate()
        {
            let rect = Rect::new(266.0 + slot as f32 * 66.0, 750.0, 58.0, 107.0);
            if self.selected == Some(*i) {
                draw_rectangle_lines(rect.x - 2.0, rect.y - 2.0, rect.w + 4.0, 88.0, 2.0, CYAN);
            }
            thumbnail(&records[*i], art, Rect::new(rect.x, rect.y, rect.w, 84.0));
            badge(&format!("x{count}"), rect.x + 28.0, rect.y + 65.0, CYAN);
            draw_centered(
                &format!("G{}", field(&records[*i], "grade")),
                rect.x,
                rect.y + 103.0,
                rect.w,
                13.0,
                MUTED,
            );
            if enabled && rect.contains(mouse) {
                if is_mouse_button_pressed(MouseButton::Left) {
                    self.selected = Some(*i);
                }
                if is_mouse_button_pressed(MouseButton::Right) {
                    self.deck.remove(*i);
                    self.status = "Removed a copy from your deck.".into();
                }
            }
        }
        draw_text(
            "Inspect a card to add / remove copies. Right click a deck card to remove.",
            266.0,
            882.0,
            12.0,
            MUTED,
        );
    }

    fn draw_preview(&mut self, records: &[Value], art: &CardArt, mouse: Vec2, enabled: bool) {
        draw_round_panel(1090.0, 114.0, 326.0, 538.0, PANEL, PANEL_LIGHT);
        draw_text("CARD DETAILS", 1106.0, 140.0, 14.0, MUTED);
        let Some(i) = self.selected else {
            return;
        };
        let card = &records[i];
        let accent = nation_color(nation(card));
        let image = Rect::new(1165.0, 155.0, 176.0, 256.0);
        thumbnail(card, art, image);
        if enabled && image.contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
            self.details = true;
        }
        draw_wrapped(
            field(card, "card_name"),
            1106.0,
            434.0,
            294.0,
            19.0,
            CREAM,
            2,
        );
        draw_centered(
            &format!(
                "{}  /  G{}  /  {} PWR",
                field(card, "clan"),
                field(card, "grade"),
                field(card, "power")
            ),
            1106.0,
            481.0,
            294.0,
            13.0,
            accent,
        );
        draw_centered(
            &format!(
                "{}    {}",
                field(card, "card_number"),
                field(card, "card_trigger")
            ),
            1106.0,
            502.0,
            294.0,
            12.0,
            MUTED,
        );
        draw_wrapped(&plain_rules(card), 1106.0, 522.0, 294.0, 12.0, MUTED, 3);
        if control(
            "READ FULL CARD",
            Rect::new(1110.0, 571.0, 286.0, 25.0),
            false,
            MUTED,
            mouse,
            enabled,
        ) {
            self.details = true;
        }
        let copies = self.deck.copies(i, records);
        if control(
            "-",
            Rect::new(1110.0, 607.0, 46.0, 30.0),
            false,
            RED,
            mouse,
            enabled && self.deck.cards.contains(&i),
        ) {
            self.deck.remove(i);
            self.status = "Removed a copy from your deck.".into();
        }
        draw_centered(&format!("{copies} / 4"), 1163.0, 629.0, 66.0, 17.0, CREAM);
        if control(
            "+ ADD",
            Rect::new(1240.0, 607.0, 156.0, 30.0),
            false,
            CYAN,
            mouse,
            enabled,
        ) {
            self.status = match self.deck.add(i, records) {
                Ok(()) => format!("Added {}.", field(card, "card_name")),
                Err(reason) => reason.into(),
            };
        }
    }

    fn draw_statistics(&mut self, records: &[Value], mouse: Vec2, enabled: bool) -> bool {
        draw_round_panel(1090.0, 666.0, 326.0, 230.0, PANEL, PANEL_LIGHT);
        draw_text("DECK DATA", 1106.0, 691.0, 16.0, CREAM);
        let triggers = self
            .deck
            .cards
            .iter()
            .filter(|i| is_trigger(&records[**i]))
            .count();
        draw_text(
            &format!("TRIGGERS  {triggers} / 16"),
            1254.0,
            691.0,
            13.0,
            if triggers == 16 { GREEN } else { MUTED },
        );
        for grade in 0..=4 {
            let count = self
                .deck
                .cards
                .iter()
                .filter(|i| number(&records[**i], "grade") == grade)
                .count();
            let x = 1110.0 + grade as f32 * 57.0;
            draw_rectangle(x, 708.0, 41.0, 42.0, PANEL_LIGHT);
            let height = (count as f32 * 1.9).min(42.0);
            draw_rectangle(
                x,
                750.0 - height,
                41.0,
                height,
                Color::new(CYAN.r, CYAN.g, CYAN.b, 0.45),
            );
            draw_centered(&count.to_string(), x, 741.0, 41.0, 16.0, CREAM);
            draw_centered(&format!("G{grade}"), x, 767.0, 41.0, 12.0, MUTED);
        }
        let counts = ["Critical", "Draw", "Stand", "Heal"].map(|kind| {
            self.deck
                .cards
                .iter()
                .filter(|i| field(&records[**i], "card_trigger").contains(kind))
                .count()
        });
        draw_text(
            &format!(
                "CRIT {}     DRAW {}     STAND {}     HEAL {}",
                counts[0], counts[1], counts[2], counts[3]
            ),
            1107.0,
            791.0,
            12.0,
            MUTED,
        );
        let validation = self.deck.validation(records);
        clipped_text(
            validation.err().unwrap_or("Ready for Quick Fight"),
            1107.0,
            815.0,
            288.0,
            13.0,
            if validation.is_ok() { GREEN } else { GOLD },
        );
        let ready = enabled && validation.is_ok();
        let save = control(
            "SAVE DECK",
            Rect::new(1107.0, 837.0, 137.0, 42.0),
            false,
            CYAN,
            mouse,
            ready,
        );
        let fight = control(
            "SAVE & FIGHT",
            Rect::new(1254.0, 837.0, 144.0, 42.0),
            false,
            GREEN,
            mouse,
            ready,
        );
        if save || fight {
            match self.deck.save(records) {
                Ok(()) => {
                    self.saved = Some(EditedDeck {
                        cards: self.deck.cards.clone(),
                    });
                    self.status = "Deck saved for Quick Fight.".into();
                    return fight;
                }
                Err(error) => self.status = format!("Could not save deck: {error}"),
            }
        }
        false
    }
}

struct CatalogRow {
    label: String,
    cards: Vec<usize>,
}

fn catalog_rows(indices: &[usize], records: &[Value], sort: CardSort) -> Vec<CatalogRow> {
    let mut rows: Vec<CatalogRow> = Vec::new();
    for i in indices {
        let card = &records[*i];
        let label = if sort == CardSort::Nation {
            format!(
                "{}   /   {}   /   GRADE {}",
                nation(card).to_uppercase(),
                field(card, "clan").to_uppercase(),
                field(card, "grade")
            )
        } else {
            sort.label().to_uppercase()
        };
        if rows
            .last()
            .is_none_or(|row| row.label != label || row.cards.len() == 5)
        {
            rows.push(CatalogRow {
                label,
                cards: Vec::new(),
            });
        }
        rows.last_mut().unwrap().cards.push(*i);
    }
    rows
}

fn nation_color(nation: &str) -> Color {
    match nation {
        "United Sanctuary" => Color::new(0.94, 0.78, 0.40, 1.0),
        "Dragon Empire" => Color::new(0.98, 0.40, 0.35, 1.0),
        "Star Gate" => Color::new(0.66, 0.78, 0.96, 1.0),
        "Dark Zone" => Color::new(0.75, 0.55, 0.96, 1.0),
        "Magallanica" => Color::new(0.35, 0.71, 0.98, 1.0),
        "Zoo" => GREEN,
        _ => MUTED,
    }
}

fn control(
    label: &str,
    rect: Rect,
    active: bool,
    accent: Color,
    mouse: Vec2,
    enabled: bool,
) -> bool {
    let hover = enabled && rect.contains(mouse);
    let color = if enabled {
        accent
    } else {
        MUTED.with_alpha(0.35)
    };
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if active || hover {
            Color::new(accent.r * 0.17, accent.g * 0.17, accent.b * 0.17, 1.0)
        } else {
            PANEL
        },
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        if active || hover { color } else { PANEL_LIGHT },
    );
    draw_centered(
        label,
        rect.x,
        rect.y + rect.h * 0.69,
        rect.w,
        if rect.h < 33.0 { 13.0 } else { 14.0 },
        if active || hover { CREAM } else { color },
    );
    hover && is_mouse_button_pressed(MouseButton::Left)
}

fn badge(text: &str, x: f32, y: f32, color: Color) {
    draw_rectangle(x, y, 33.0, 20.0, INK);
    draw_centered(text, x, y + 15.0, 33.0, 13.0, color);
}

fn clipped_text(text: &str, x: f32, y: f32, width: f32, size: f32, color: Color) {
    let mut result = text.to_owned();
    if measure_text(&result, None, size as u16, 1.0).width > width {
        while measure_text(&format!("{result}..."), None, size as u16, 1.0).width > width
            && !result.is_empty()
        {
            result.pop();
        }
        result.push_str("...");
    }
    draw_text(&result, x, y, size, color);
}

fn thumbnail(card: &Value, art: &CardArt, rect: Rect) {
    if let Some(texture) = art.printing(card) {
        // Fit using physical viewport proportions so wide windows never stretch card art.
        let viewport_scale = vec2(screen_width() / WIDTH, screen_height() / HEIGHT);
        let fit = (rect.w * viewport_scale.x / texture.width())
            .min(rect.h * viewport_scale.y / texture.height());
        let size = vec2(
            texture.width() * fit / viewport_scale.x,
            texture.height() * fit / viewport_scale.y,
        );
        draw_texture_ex(
            texture,
            rect.x + (rect.w - size.x) * 0.5,
            rect.y + (rect.h - size.y) * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
    } else {
        draw_round_panel(rect.x, rect.y, rect.w, rect.h, PANEL_LIGHT, MUTED);
        draw_centered(
            "NO IMAGE",
            rect.x,
            rect.y + rect.h * 0.5,
            rect.w,
            12.0,
            MUTED,
        );
    }
}

fn plain_rules(card: &Value) -> String {
    field(card, "text")
        .replace("<br>", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&lt", "<")
        .replace("&gt", ">")
        .replace("&amp;", "&")
}

fn draw_details(card: &Value, art: &CardArt) {
    draw_rectangle(0.0, 0.0, WIDTH, HEIGHT, Color::new(0.0, 0.01, 0.025, 0.94));
    draw_round_panel(100.0, 98.0, 1240.0, 764.0, PANEL, CYAN);
    thumbnail(card, art, Rect::new(138.0, 176.0, 310.0, 451.0));
    draw_wrapped(
        field(card, "card_name"),
        475.0,
        190.0,
        810.0,
        28.0,
        CREAM,
        2,
    );
    draw_centered(
        &format!(
            "{} / {} / Grade {} / Power {} / Shield {}",
            field(card, "card_number"),
            field(card, "clan"),
            field(card, "grade"),
            field(card, "power"),
            field(card, "shield")
        ),
        475.0,
        268.0,
        810.0,
        17.0,
        CYAN,
    );
    draw_wrapped(&plain_rules(card), 487.0, 317.0, 780.0, 18.0, CREAM, 23);
    draw_wrapped(field(card, "product"), 130.0, 690.0, 325.0, 16.0, MUTED, 3);
    draw_wrapped(field(card, "flavor"), 130.0, 775.0, 1170.0, 16.0, GOLD, 3);
}

fn clan_icons() -> HashMap<&'static str, Texture2D> {
    [
        (
            "Angel Feather",
            super::embedded_texture(include_bytes!("../../assets/clans/angel-feather.png")),
        ),
        (
            "Aqua Force",
            super::embedded_texture(include_bytes!("../../assets/clans/aqua-force.png")),
        ),
        (
            "Bermuda Triangle",
            super::embedded_texture(include_bytes!("../../assets/clans/bermuda-triangle.png")),
        ),
        (
            "Dark Irregulars",
            super::embedded_texture(include_bytes!("../../assets/clans/dark-irregulars.png")),
        ),
        (
            "Dimension Police",
            super::embedded_texture(include_bytes!("../../assets/clans/dimension-police.png")),
        ),
        (
            "Genesis",
            super::embedded_texture(include_bytes!("../../assets/clans/genesis.png")),
        ),
        (
            "Gold Paladin",
            super::embedded_texture(include_bytes!("../../assets/clans/gold-paladin.png")),
        ),
        (
            "Granblue",
            super::embedded_texture(include_bytes!("../../assets/clans/granblue.png")),
        ),
        (
            "Great Nature",
            super::embedded_texture(include_bytes!("../../assets/clans/great-nature.png")),
        ),
        (
            "Kagero",
            super::embedded_texture(include_bytes!("../../assets/clans/kagero.png")),
        ),
        (
            "Link Joker",
            super::embedded_texture(include_bytes!("../../assets/clans/link-joker.png")),
        ),
        (
            "Megacolony",
            super::embedded_texture(include_bytes!("../../assets/clans/megacolony.png")),
        ),
        (
            "Murakumo",
            super::embedded_texture(include_bytes!("../../assets/clans/murakumo.png")),
        ),
        (
            "Narukami",
            super::embedded_texture(include_bytes!("../../assets/clans/narukami.png")),
        ),
        (
            "Neo Nectar",
            super::embedded_texture(include_bytes!("../../assets/clans/neo-nectar.png")),
        ),
        (
            "Nova Grappler",
            super::embedded_texture(include_bytes!("../../assets/clans/nova-grappler.png")),
        ),
        (
            "Nubatama",
            super::embedded_texture(include_bytes!("../../assets/clans/nubatama.png")),
        ),
        (
            "Oracle Think Tank",
            super::embedded_texture(include_bytes!("../../assets/clans/oracle-think-tank.png")),
        ),
        (
            "Pale Moon",
            super::embedded_texture(include_bytes!("../../assets/clans/pale-moon.png")),
        ),
        (
            "Royal Paladin",
            super::embedded_texture(include_bytes!("../../assets/clans/royal-paladin.png")),
        ),
        (
            "Shadow Paladin",
            super::embedded_texture(include_bytes!("../../assets/clans/shadow-paladin.png")),
        ),
        (
            "Spike Brothers",
            super::embedded_texture(include_bytes!("../../assets/clans/spike-brothers.png")),
        ),
        (
            "Tachikaze",
            super::embedded_texture(include_bytes!("../../assets/clans/tachikaze.png")),
        ),
    ]
    .into_iter()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn grouped_rows_keep_boundaries_and_every_card() {
        let records = (0..17).map(|i| json!({"country":"Dragon Empire", "clan": if i < 8 {"Kagero"} else {"Narukami"}, "grade": "1"})).collect::<Vec<_>>();
        let indices = (0..17).collect::<Vec<_>>();
        let rows = catalog_rows(&indices, &records, CardSort::Nation);
        assert!(rows.iter().all(|r| r.cards.len() <= 5));
        assert_eq!(
            rows.iter()
                .flat_map(|r| r.cards.iter().copied())
                .collect::<Vec<_>>(),
            indices
        );
        assert!(rows.iter().all(|r| r
            .cards
            .iter()
            .all(|i| field(&records[*i], "clan") == field(&records[r.cards[0]], "clan"))));
    }
}
