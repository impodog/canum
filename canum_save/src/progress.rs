use std::collections::{BTreeSet, HashMap, HashSet};

use crate::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Charms {
    pub has_offensive: bool,
    pub has_defensive: bool,
    pub selected_charms: HashSet<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Progress {
    pub weapon_slots: usize,
    pub unlocked_dash: bool,
    pub selected_weapons: Vec<String>,
    pub selected_effects: HashSet<String>,
    pub selected_health: String,
    pub charms: Charms,
    pub gained_charms: BTreeSet<String>,
    pub gained_weapons: BTreeSet<String>,
    pub boss_progress: HashMap<String, BossProgress>,
    pub completed_stages: HashSet<String>,
    pub current_lobby: String,
    pub lobby_position: Vec2,
    pub coins: i32,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            weapon_slots: 1,
            unlocked_dash: false,
            selected_weapons: vec!["A_Filed".to_owned()],
            selected_effects: Default::default(),
            selected_health: "BasicHp".to_string(),
            charms: Default::default(),
            gained_charms: Default::default(),
            gained_weapons: BTreeSet::from_iter(["A_Filed".to_owned()]),
            boss_progress: Default::default(),
            completed_stages: Default::default(),
            current_lobby: "Gate".to_owned(),
            lobby_position: Vec2::new(400.0, 225.0),
            coins: 0,
        }
    }
}
impl Progress {
    pub fn boss_progress(&mut self, name: impl Into<String>) -> &mut BossProgress {
        self.boss_progress.entry(name.into()).or_default()
    }

    pub fn has_shop_item(&self, item: &canum_res::config::ShopItem) -> bool {
        use canum_res::config::ShopItem;
        match item {
            ShopItem::Charm(charm) => self.gained_charms.contains(charm),
            ShopItem::Weapon(weapon) => self.gained_weapons.contains(weapon),
        }
    }
    pub fn insert_shop_item(&mut self, item: canum_res::config::ShopItem) {
        use canum_res::config::ShopItem;
        match item {
            ShopItem::Charm(charm) => self.gained_charms.insert(charm),
            ShopItem::Weapon(weapon) => self.gained_weapons.insert(weapon),
        };
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BossProgress {
    pub defeated: bool,
    pub fail_times: usize,
    pub tasks: BTreeSet<String>,
}
