mod assets;
pub use assets::*;

mod values;
pub use values::*;

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
    #[serde(skip)]
    pub screen_rect: bevy::prelude::Rect,
    #[serde(skip)]
    pub virtual_diagonal: f32,
}
impl Default for Display {
    fn default() -> Self {
        Self {
            fullscreen: false,
            window_size: (1920, 1080),
            virtual_size: (800, 450),
            half_virtual_size: (400.0, 225.0),
            screen_size: bevy::prelude::Vec2::new(800.0, 450.0),
            screen_rect: bevy::prelude::Rect::from_center_half_size(Vec2::ZERO, vec2(400.0, 225.0)),
            virtual_diagonal: 917.878,
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
    pub text_roll_speed: f32,
    #[serde(skip)]
    pub text_roll_interval: Duration,
}
impl Default for Client {
    fn default() -> Self {
        Self {
            framerate: 100,
            frame_duration: Duration::from_secs_f32(1.0 / 100.0),
            update_freq: 64.0,
            update_duration: 1.0 / 64.0,
            text_roll_speed: 25.0,
            text_roll_interval: Duration::from_secs_f32(1.0 / 25.0),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Fonts {
    pub game_font: PathBuf,
    pub text_font: PathBuf,
    pub title_font: PathBuf,
}
impl Default for Fonts {
    fn default() -> Self {
        Self {
            game_font: "./assets/fonts/Crimson.ttf".into(),
            text_font: "./assets/fonts/terminal-grotesque.ttf".into(),
            title_font: "./assets/fonts/Hack.ttf".into(),
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
    pub fonts: Fonts,

    pub assets_path: PathBuf,
    #[serde(skip)]
    pub assets: AssetsConfig,

    pub values_path: PathBuf,
    #[serde(skip)]
    pub values: Values,
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
            config.display.screen_rect =
                bevy::prelude::Rect::from_center_size(Vec2::ZERO, config.display.screen_size);
            config.display.virtual_diagonal = config.display.screen_size.length();
            config.client.frame_duration =
                std::time::Duration::from_secs_f32(1.0 / config.client.framerate as f32);
            config.client.update_duration = 1.0 / config.client.update_freq;
            config.client.text_roll_interval =
                Duration::from_secs_f32(1.0 / config.client.text_roll_speed);
            AssetsConfig::load(&mut config.assets, config.assets_path.clone());
            Values::load(&mut config.values, config.values_path.clone());
            config
        }
        Err(err) => {
            log::warn!("Unable to find canum.toml: {err}, using default config");
            Config::default()
        }
    });
