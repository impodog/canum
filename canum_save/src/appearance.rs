use crate::*;

/// User customizable appearances.
#[derive(Debug, Serialize, Deserialize)]
pub struct Appearance {
    pub player: String,
}
impl Default for Appearance {
    fn default() -> Self {
        Self {
            player: "Cyan".to_owned(),
        }
    }
}
