use super::*;

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
    pub fn load(base: &mut AssetsConfig, path: PathBuf) {
        let base_path = path
            .parent()
            .expect("Config file should be a file with a parent directory");
        let mut config: AssetsConfig = match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => {
                    log::debug!("Assets config loaded at {path:?}");
                    config
                }
                Err(err) => {
                    panic!("Failed to parse assets config at {path:?}: {err}");
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
        base.sprites.extend(config.sprites);
        base.sounds.extend(config.sounds);
        base.tinting.extend(config.tinting);
        for (language, text) in config.text {
            base.text.entry(language).or_default().0.extend(text.0);
        }
        for sub_path in config.include {
            let sub_path = base_path.join(&sub_path);
            AssetsConfig::load(base, sub_path);
        }
    }
}
