use std::collections::HashSet;

use super::*;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Values {
    #[serde(default)]
    pub include: Vec<PathBuf>,
    #[serde(default)]
    /// Maps from boss code name to gains.
    pub boss: HashMap<String, BossDetails>,
    #[serde(default)]
    pub stage: HashMap<String, StageDetails>,
    #[serde(default)]
    pub charm: HashMap<String, CharmDetails>,
    #[serde(default)]
    pub shop: HashMap<String, ShopDetails>,
}

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
pub struct StageDetails {
    pub unlock_prereqs: HashSet<String>,
    pub full_size: Vec2,
    pub locked_size: Vec2,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CharmCost(u8);

impl CharmCost {
    /// If the charm costs a defensive slot.
    pub const fn defensive(self) -> bool {
        (self.0 & 0x1) != 0
    }
    /// If the charm costs a offensive slot.
    pub const fn offensive(self) -> bool {
        (self.0 & 0x2) != 0
    }
    /// If the charm costs a general slot.
    pub const fn general(self) -> bool {
        (self.0 & 0x4) != 0
    }
}
impl<'de> Deserialize<'de> for CharmCost {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CharmCostVisitor;
        impl<'de> serde::de::Visitor<'de> for CharmCostVisitor {
            type Value = CharmCost;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a string consisting of 'D' 'O' 'G'")
            }
            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut value = 0u8;
                for ch in v.chars() {
                    match ch {
                        'D' => value |= 0x1,
                        'O' => value |= 0x2,
                        'G' => value |= 0x4,
                        _ => {
                            return Err(serde::de::Error::invalid_value(
                                serde::de::Unexpected::Char(ch),
                                &"any of 'D' 'O' 'G'",
                            ));
                        }
                    }
                }
                Ok(CharmCost(value))
            }
        }
        deserializer.deserialize_str(CharmCostVisitor)
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct CharmDetails {
    pub cost: CharmCost,
    pub effects: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum ShopItem {
    Charm(String),
    Weapon(String),
}

impl ShopItem {
    pub fn to_name(&self) -> String {
        match self {
            Self::Charm(name) => format!("Charm_{name}"),
            Self::Weapon(name) => format!("Weapon_{name}"),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ShopItemDetails {
    pub price: i32,
    pub item: ShopItem,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ShopDetails {
    pub items: Vec<ShopItemDetails>,
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
        for (name, details) in values.stage {
            base.stage.insert(name, details);
        }
        for (name, details) in values.charm {
            base.charm.insert(name, details);
        }
        for sub_path in values.include {
            let sub_path = base_path.join(&sub_path);
            Values::load(base, sub_path);
        }
    }
}
