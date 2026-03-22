use crate::*;

/// User keyboard binding configurations.
#[derive(Debug, Serialize, Deserialize)]
pub struct Keyboard {
    pub move_right: KeyCode,
    pub move_up: KeyCode,
    pub move_left: KeyCode,
    pub move_down: KeyCode,
    pub dash: KeyCode,
    pub primary_attack: KeyCode,
    pub secondary_attack: KeyCode,
}
impl Default for Keyboard {
    fn default() -> Self {
        Self {
            move_right: KeyCode::ArrowRight,
            move_up: KeyCode::ArrowUp,
            move_left: KeyCode::ArrowLeft,
            move_down: KeyCode::ArrowDown,
            dash: KeyCode::KeyX,
            primary_attack: KeyCode::KeyC,
            secondary_attack: KeyCode::KeyZ,
        }
    }
}
