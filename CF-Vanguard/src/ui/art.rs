//! Resolves cached card artwork independently of the directory that launched the game.

use macroquad::prelude::{FilterMode, Image, ImageFormat, Texture2D};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

const TEXTURE_CACHE_LIMIT: usize = 150;

pub(super) struct CardArt {
    roots: Vec<PathBuf>,
    paths_by_name: HashMap<String, Vec<PathBuf>>,
    textures: HashMap<String, Texture2D>,
    unavailable: HashSet<String>,
}

impl CardArt {
    pub(super) fn new(records: &[Value]) -> Self {
        let exe = std::env::current_exe().ok();
        let cwd = std::env::current_dir().ok();
        Self::from_layout(
            records,
            exe.as_deref(),
            Path::new(env!("CARGO_MANIFEST_DIR")),
            cwd.as_deref(),
        )
    }

    fn from_layout(
        records: &[Value],
        executable: Option<&Path>,
        project_root: &Path,
        working_directory: Option<&Path>,
    ) -> Self {
        let mut paths_by_name: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for record in records {
            if let (Some(name), Some(path)) =
                (record["card_name"].as_str(), record["image_file"].as_str())
            {
                paths_by_name
                    .entry(normalize_card_name(name))
                    .or_default()
                    .push(PathBuf::from(path));
                paths_by_name.insert(printing_key(record), vec![PathBuf::from(path)]);
            }
        }

        Self {
            roots: artwork_roots(executable, project_root, working_directory),
            paths_by_name,
            textures: HashMap::new(),
            unavailable: HashSet::new(),
        }
    }

    pub(super) fn preload(&mut self, names: &[String]) {
        let keys: HashSet<_> = names.iter().map(|name| normalize_card_name(name)).collect();
        self.preload_keys(keys);
    }

    pub(super) fn preload_records(&mut self, indices: &[usize], records: &[Value]) {
        let keys = indices
            .iter()
            .filter_map(|i| records.get(*i))
            .map(printing_key)
            .collect();
        self.preload_keys(keys);
    }

    pub(super) fn printing(&self, record: &Value) -> Option<&Texture2D> {
        self.textures.get(&printing_key(record))
    }

    fn preload_keys(&mut self, keys: HashSet<String>) {
        if self.textures.len() + keys.len() > TEXTURE_CACHE_LIMIT {
            self.textures.retain(|key, _| keys.contains(key));
        }
        for key in keys {
            if self.textures.contains_key(&key) || self.unavailable.contains(&key) {
                continue;
            }
            if let Some(image) = self.load_image(&key) {
                let texture = Texture2D::from_image(&image);
                texture.set_filter(FilterMode::Linear);
                self.textures.insert(key, texture);
            } else {
                self.unavailable.insert(key);
            }
        }
    }

    pub(super) fn textures(&self) -> &HashMap<String, Texture2D> {
        &self.textures
    }

    fn load_image(&self, key: &str) -> Option<Image> {
        // Different printings can share a name. Try the next printing if its file is unavailable.
        self.paths_by_name.get(key)?.iter().find_map(|relative| {
            self.roots.iter().find_map(|root| {
                let bytes = std::fs::read(root.join(relative)).ok()?;
                Image::from_file_with_format(&bytes, Some(ImageFormat::Png)).ok()
            })
        })
    }
}

fn printing_key(record: &Value) -> String {
    format!("printing:{}", record["card_number"].as_str().unwrap_or(""))
}

fn artwork_roots(
    executable: Option<&Path>,
    project_root: &Path,
    working_directory: Option<&Path>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(directory) = executable.and_then(Path::parent) {
        // A copied executable can ship with assets beside it, or in the app's Resources.
        candidates.push(directory.to_owned());
        if directory.file_name().is_some_and(|name| name == "MacOS") {
            if let Some(contents) = directory.parent() {
                candidates.push(contents.join("Resources"));
            }
        }

        // Standard Cargo builds can still locate the assets if the whole checkout was moved.
        if let Some(target) = directory
            .ancestors()
            .find(|path| path.file_name().is_some_and(|name| name == "target"))
        {
            if let Some(checkout) = target.parent() {
                candidates.push(checkout.to_owned());
            }
        }
    }
    candidates.push(project_root.to_owned());
    candidates.extend(working_directory.map(Path::to_owned));

    let mut roots = Vec::new();
    for candidate in candidates {
        if candidate.join("assets/card_images").is_dir() && !roots.contains(&candidate) {
            roots.push(candidate);
        }
    }
    roots
}

pub(super) fn normalize_card_name(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn editor_printings_resolve_their_exact_image_paths() {
        let records = [
            json!({"card_name": "Dragonic Overlord", "card_number": "BT01/004EN", "image_file": "assets/card_images/BT01_004EN.png"}),
            json!({"card_name": "Dragonic Overlord", "card_number": "BT15/004EN", "image_file": "assets/card_images/BT15_004EN.png"}),
        ];
        let art = CardArt::new(&records);
        for record in &records {
            assert_eq!(
                art.paths_by_name[&printing_key(record)],
                vec![PathBuf::from(record["image_file"].as_str().unwrap())]
            );
            assert!(art.load_image(&printing_key(record)).is_some());
        }
        assert_ne!(printing_key(&records[0]), printing_key(&records[1]));
    }

    #[test]
    fn local_card_art_decodes_with_an_unrelated_working_directory() {
        let records = [json!({
            "card_name": "King of Knights, Alfred",
            "image_file": "assets/card_images/BT01_001EN.png",
        })];
        let card_art = CardArt::from_layout(
            &records,
            None,
            Path::new(env!("CARGO_MANIFEST_DIR")),
            Some(Path::new("/unrelated/working/directory")),
        );

        let image = card_art
            .load_image(&normalize_card_name("King of Knights Alfred"))
            .expect("the bundled card should load from the project");
        assert!(image.width() > 0 && image.height() > 0);
    }

    #[test]
    fn missing_printing_can_fall_back_to_another_printing() {
        let records = [
            json!({
                "card_name": "King of Knights, Alfred",
                "image_file": "assets/card_images/no-such-printing.png",
            }),
            json!({
                "card_name": "King of Knights, Alfred",
                "image_file": "assets/card_images/BT01_001EN.png",
            }),
        ];
        let card_art = CardArt::new(&records);

        assert!(card_art
            .load_image(&normalize_card_name("King of Knights, Alfred"))
            .is_some());
    }
}
