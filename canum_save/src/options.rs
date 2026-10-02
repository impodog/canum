use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Options {
    pub always_show_health: bool,
    pub show_aim: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            always_show_health: false,
            show_aim: true,
        }
    }
}
