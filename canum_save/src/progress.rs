use std::collections::HashMap;

use crate::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Progress {
    pub weapon_slots: usize,
    pub unlocked_dash: bool,
    pub selected_weapons: Vec<String>,
    pub selected_health: String,
    pub boss_progress: HashMap<String, BossProgress>,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            weapon_slots: 1,
            unlocked_dash: false,
            selected_weapons: vec!["Filed".to_owned()],
            selected_health: "BasicHp".to_string(),
            boss_progress: Default::default(),
        }
    }
}
impl Progress {
    pub fn boss_progress(&mut self, name: impl Into<String>) -> &mut BossProgress {
        self.boss_progress.entry(name.into()).or_default()
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BossProgress {
    pub defeated: bool,
}
