//! Official anime fighter portraits shared by story and quick fights.
use super::embedded_texture;
use macroquad::prelude::Texture2D;

pub(super) struct FighterArt {
    portraits: Vec<(&'static str, Texture2D)>,
    standing: Vec<(&'static str, Texture2D)>,
}

impl FighterArt {
    pub(super) fn new() -> Self {
        Self {
            standing: vec![
                (
                    "aichi",
                    embedded_texture(include_bytes!("../../assets/fighters/aichi-standing.png")),
                ),
                (
                    "kai",
                    embedded_texture(include_bytes!("../../assets/fighters/kai-standing.png")),
                ),
                (
                    "morikawa",
                    embedded_texture(include_bytes!(
                        "../../assets/fighters/morikawa-standing.png"
                    )),
                ),
                (
                    "kamui",
                    embedded_texture(include_bytes!("../../assets/fighters/kamui-standing.png")),
                ),
                (
                    "misaki",
                    embedded_texture(include_bytes!("../../assets/fighters/misaki-standing.png")),
                ),
                (
                    "ren",
                    embedded_texture(include_bytes!("../../assets/fighters/ren-standing.png")),
                ),
                (
                    "asaka",
                    embedded_texture(include_bytes!("../../assets/fighters/asaka-standing.png")),
                ),
                (
                    "leon",
                    embedded_texture(include_bytes!("../../assets/fighters/leon-standing.png")),
                ),
                (
                    "naoki",
                    embedded_texture(include_bytes!("../../assets/fighters/naoki-standing.png")),
                ),
                (
                    "takuto",
                    embedded_texture(include_bytes!("../../assets/fighters/takuto-standing.png")),
                ),
            ],
            portraits: vec![
                (
                    "aichi",
                    embedded_texture(include_bytes!("../../assets/fighters/aichi-icon.png")),
                ),
                (
                    "kai",
                    embedded_texture(include_bytes!("../../assets/fighters/kai-icon.png")),
                ),
                (
                    "morikawa",
                    embedded_texture(include_bytes!("../../assets/fighters/morikawa-icon.png")),
                ),
                (
                    "kamui",
                    embedded_texture(include_bytes!("../../assets/fighters/kamui-icon.png")),
                ),
                (
                    "misaki",
                    embedded_texture(include_bytes!("../../assets/fighters/misaki-icon.png")),
                ),
                (
                    "ren",
                    embedded_texture(include_bytes!("../../assets/fighters/ren-icon.png")),
                ),
                (
                    "asaka",
                    embedded_texture(include_bytes!("../../assets/fighters/asaka-icon.png")),
                ),
                (
                    "leon",
                    embedded_texture(include_bytes!("../../assets/fighters/leon-icon.png")),
                ),
                (
                    "naoki",
                    embedded_texture(include_bytes!("../../assets/fighters/naoki-icon.png")),
                ),
                (
                    "takuto",
                    embedded_texture(include_bytes!("../../assets/fighters/takuto-icon.png")),
                ),
            ],
        }
    }

    pub(super) fn cutin(&self, name: &str) -> Option<&Texture2D> {
        let name = name.to_lowercase();
        self.standing
            .iter()
            .find(|(key, _)| {
                name.split(|c: char| !c.is_alphabetic())
                    .any(|word| word == *key)
            })
            .map(|(_, image)| image)
    }

    pub(super) fn portrait(&self, name: &str) -> Option<&Texture2D> {
        let name = name.to_lowercase();
        self.portraits
            .iter()
            .find(|(key, _)| {
                name.split(|c: char| !c.is_alphabetic())
                    .any(|word| word == *key)
            })
            .map(|(_, image)| image)
    }
}
