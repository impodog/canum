use crate::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Progress {
    pub weapon_slots: usize,
    pub unlocked_dash: bool,
    pub selected_weapons: Vec<String>,
    pub selected_health: String,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            weapon_slots: 1,
            unlocked_dash: false,
            selected_weapons: vec!["Filed".to_owned()],
            selected_health: "Basic".to_string(),
        }
    }
}
