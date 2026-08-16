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
    pub confirm: KeyCode,
    pub shop: KeyCode,
    pub equip: KeyCode,
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
            confirm: KeyCode::Enter,
            shop: KeyCode::KeyS,
            equip: KeyCode::KeyE,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gamepad {
    pub dash: GamepadButton,
    pub primary_attack: GamepadButton,
    pub secondary_attack: GamepadButton,
    pub confirm: GamepadButton,
    pub cancel: GamepadButton,
    pub shop: GamepadButton,
    pub equip: GamepadButton,
}
impl Default for Gamepad {
    fn default() -> Self {
        Self {
            dash: GamepadButton::RightTrigger2,
            primary_attack: GamepadButton::West,
            secondary_attack: GamepadButton::North,
            confirm: GamepadButton::South,
            cancel: GamepadButton::East,
            shop: GamepadButton::RightThumb,
            equip: GamepadButton::LeftThumb,
        }
    }
}
