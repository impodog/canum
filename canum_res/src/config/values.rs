use super::*;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct BossDetails {
    /// Gain coins from getting a completion mark.
    pub gains: HashMap<String, i32>,
}

impl BossDetails {
    pub fn merge(&mut self, other: BossDetails) {
        self.gains.extend(other.gains);
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Values {
    #[serde(default)]
    pub include: Vec<PathBuf>,
    #[serde(default)]
    /// Maps from boss code name to gains.
    pub boss: HashMap<String, BossDetails>,
}

impl Values {
    pub fn load(base: &mut Values, path: PathBuf) {
        let base_path = path
            .parent()
            .expect("Config file should be a file with a parent directory");
        let values: Values = match std::fs::read_to_string(&path) {
            Ok(content) => match ron::from_str(&content) {
                Ok(values) => {
                    log::debug!("Gameplay values loaded at {path:?}");
                    values
                }
                Err(err) => {
                    panic!("Failed to parse gameplay values at {path:?}: {err}");
                }
            },
            Err(err) => {
                log::error!("Unable to read gameplay values {path:?}(skipped): {err}");
                Default::default()
            }
        };
        for (name, details) in values.boss {
            base.boss.entry(name).or_default().merge(details);
        }
        for sub_path in values.include {
            let sub_path = base_path.join(&sub_path);
            Values::load(base, sub_path);
        }
    }
}
