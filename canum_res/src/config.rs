use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::LazyLock, time::Duration};

#[derive(Serialize, Deserialize, Debug)]
pub struct Display {
    pub fullscreen: bool,
    pub window_size: (u32, u32),
    pub virtual_size: (u32, u32),
    #[serde(skip)]
    pub half_virtual_size: (f32, f32),
    #[serde(skip)]
    pub screen_size: bevy::prelude::Vec2,
}
impl Default for Display {
    fn default() -> Self {
        Self {
            fullscreen: false,
            window_size: (1920, 1080),
            virtual_size: (800, 450),
            half_virtual_size: (400.0, 225.0),
            screen_size: bevy::prelude::Vec2::new(800.0, 450.0),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Client {
    pub framerate: u32,
    #[serde(skip)]
    pub frame_duration: Duration,
    pub update_freq: f32,
    #[serde(skip)]
    pub update_duration: f32,
}
impl Default for Client {
    fn default() -> Self {
        Self {
            framerate: 100,
            frame_duration: Duration::from_secs_f32(1.0 / 60.0),
            update_freq: 64.0,
            update_duration: 1.0 / 64.0,
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

#[derive(Deserialize, Debug, Clone, Default)]
pub struct SoundDetails {
    /// Sound file path.
    pub path: PathBuf,
    #[serde(default = "return_1_0")]
    pub volume: f32,
    /// The sound loops between the point to end. If None, the sound only plays once.
    #[serde(default)]
    pub loop_point: Option<f32>,
}
const fn return_1_0() -> f32 {
    1.0
}

#[derive(Deserialize, Debug, Clone, Default, Deref, DerefMut)]
pub struct LanguageConfig(pub HashMap<String, String>);

/// Configuration for assets in the game.
#[derive(Default, Deserialize, Debug, Clone)]
pub struct AssetsConfig {
    /// Any child configurations that will be merged into this config.
    #[serde(default)]
    pub include: Vec<PathBuf>,
    /// Map from aliases to the actual sprite.
    #[serde(default)]
    pub sprites: HashMap<String, Vec<SpriteAtlas>>,
    /// Defines tinting style aliases.
    #[serde(default)]
    pub tinting: HashMap<String, Color>,
    /// Map from aliases to sound details.
    #[serde(default)]
    pub sounds: HashMap<String, SoundDetails>,
    /// Map from language to text.
    #[serde(default)]
    pub text: HashMap<String, LanguageConfig>,
}

impl AssetsConfig {
    pub fn load(path: PathBuf) -> Self {
        let base_path = path
            .parent()
            .expect("Config file should be a file with a parent directory");
        let mut config: AssetsConfig = match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => {
                    log::info!("Assets config loaded at {path:?}");
                    config
                }
                Err(err) => {
                    panic!("Failed to parse assets config: {err}");
                }
            },
            Err(err) => {
                log::error!("Unable to read assets config {path:?}(skipped): {err}");
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
                    log::error!("Unable to read configured image path: {new_path:?}(skipped)");
                }
            }
            *list = new_list;
        }
        for sound in config.sounds.values_mut() {
            let new_path = base_path.join(&sound.path);
            if let Ok(new_path) = new_path.canonicalize() {
                sound.path = new_path;
            } else {
                log::error!("Unable to read configured sound path: {new_path:?}(skipped)")
            }
        }
        for sub_path in config.include.drain(..) {
            let sub_path = base_path.join(&sub_path);
            let AssetsConfig {
                include: _,
                sprites,
                tinting,
                sounds,
                text,
            } = AssetsConfig::load(sub_path);
            config.sprites.extend(sprites);
            config.tinting.extend(tinting);
            config.sounds.extend(sounds);
            for (language, content) in text.into_iter() {
                config.text.entry(language).or_default().extend(content.0);
            }
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
    pub assets_path: PathBuf,
    #[serde(skip)]
    pub assets: AssetsConfig,
    #[serde(default)]
    pub fonts: Fonts,
}
pub static CONFIG: LazyLock<Config> =
    LazyLock::new(|| match std::fs::read_to_string("canum.toml") {
        Ok(content) => {
            let mut config: Config =
                toml::from_str(content.as_str()).expect("canum.toml failed to parse");
            config.display.half_virtual_size.0 = config.display.virtual_size.0 as f32 * 0.5;
            config.display.half_virtual_size.1 = config.display.virtual_size.1 as f32 * 0.5;
            config.display.screen_size = bevy::prelude::Vec2::new(
                config.display.virtual_size.0 as f32,
                config.display.virtual_size.1 as f32,
            );
            config.client.frame_duration =
                std::time::Duration::from_secs_f32(1.0 / config.client.framerate as f32);
            config.client.update_duration = 1.0 / config.client.update_freq;
            config.assets = AssetsConfig::load(config.assets_path.clone());
            config
        }
        Err(err) => {
            log::warn!("Unable to find canum.toml: {err}, using default config");
            Config::default()
        }
    });
