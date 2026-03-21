use bevy::color::Color;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::LazyLock, time::Duration};

#[derive(Serialize, Deserialize, Debug)]
pub struct Display {
    pub fullscreen: bool,
    pub window_size: (u32, u32),
    pub virtual_size: (u32, u32),
}
impl Default for Display {
    fn default() -> Self {
        Self {
            fullscreen: false,
            window_size: (1920, 1080),
            virtual_size: (800, 450),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Client {
    pub framerate: u32,
    #[serde(skip)]
    pub frame_duration: Duration,
}
impl Default for Client {
    fn default() -> Self {
        Self {
            framerate: 60,
            frame_duration: Duration::from_secs_f32(1.0 / 60.0),
        }
    }
}

/// One sprite atlas that will be played repeatedly in the game.
#[derive(Deserialize, Debug, Clone)]
pub struct SpriteAtlas {
    pub path: PathBuf,
    #[serde(default)]
    pub offset: (u32, u32),
    #[serde(default = "return_default_size")]
    pub size: (u32, u32),
    #[serde(default = "return_1")]
    pub count: u32,
    /// Milliseconds between switching frames.
    #[serde(default = "return_750")]
    pub interval: u32,
}
const fn return_default_size() -> (u32, u32) {
    (32, 32)
}
const fn return_1() -> u32 {
    1
}
const fn return_750() -> u32 {
    750
}

/// Configuration for sprites in the game.
#[derive(Default, Deserialize, Debug, Clone)]
pub struct Sprites {
    /// Any child configurations that will be merged into this config.
    #[serde(default)]
    pub include: Vec<PathBuf>,
    /// Map from aliases to the actual sprite.
    #[serde(default)]
    pub sprites: HashMap<String, Vec<SpriteAtlas>>,
    /// Defines tinting style aliases.
    #[serde(default)]
    pub tinting: HashMap<String, Color>,
}

impl Sprites {
    pub fn load(path: PathBuf) -> Self {
        let base_path = path
            .parent()
            .expect("Config file should be a file with a parent directory");
        let mut config: Sprites = match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => {
                    log::info!("Sprite config loaded at {path:?}");
                    config
                }
                Err(err) => {
                    panic!("Failed to parse sprite config: {err}");
                }
            },
            Err(err) => {
                log::error!("Unable to read sprite config {path:?}(skipped): {err}");
                Default::default()
            }
        };
        for list in config.sprites.values_mut() {
            let mut new_list = Vec::new();
            for mut atlas in list.drain(..) {
                let new_path = base_path.join(&atlas.path);
                if let Ok(new_path) = new_path.canonicalize() {
                    atlas.path = new_path;
                    new_list.push(atlas);
                } else {
                    log::error!("Unable to read configurated image path: {new_path:?}(skipped)");
                }
            }
            *list = new_list;
        }
        for sub_path in config.include.drain(..) {
            let sub_path = base_path.join(&sub_path);
            let Sprites {
                sprites, tinting, ..
            } = Sprites::load(sub_path);
            config.sprites.extend(sprites);
            config.tinting.extend(tinting)
        }
        config
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Fonts {
    pub ui_font: PathBuf,
    pub text_font: PathBuf,
}
impl Default for Fonts {
    fn default() -> Self {
        Self {
            ui_font: "./assets/fonts/Crimson.ttf".into(),
            text_font: "./assets/fonts/JetbrainsMono.ttf".into(),
        }
    }
}

#[derive(Deserialize, Default, Debug)]
pub struct Config {
    #[serde(default)]
    pub display: Display,
    #[serde(default)]
    pub client: Client,
    #[serde(default)]
    pub sprites_path: PathBuf,
    #[serde(skip)]
    pub sprites: Sprites,
    #[serde(default)]
    pub fonts: Fonts,
}
pub static CONFIG: LazyLock<Config> =
    LazyLock::new(|| match std::fs::read_to_string("canum.toml") {
        Ok(content) => {
            let mut config: Config =
                toml::from_str(content.as_str()).expect("canum.toml failed to parse");
            config.client.frame_duration =
                std::time::Duration::from_secs_f32(1.0 / config.client.framerate as f32);
            config.sprites = Sprites::load(config.sprites_path.clone());
            log::info!("Loaded sprites: {:?}", config.sprites);
            config
        }
        Err(err) => {
            log::warn!("Unable to find canum.toml: {err}, using default config");
            Config::default()
        }
    });
