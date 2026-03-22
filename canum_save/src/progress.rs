use crate::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Progress {
    pub weapon_slots: usize,
}

impl Default for Progress {
    fn default() -> Self {
        Self { weapon_slots: 2 }
    }
}
