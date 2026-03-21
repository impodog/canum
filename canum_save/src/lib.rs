//! Loads and handles user save profiles.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct CanumSavePlugin;
impl Plugin for CanumSavePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, read_save);
        app.add_systems(Last, write_save_on_exit);
    }
}

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

/// User keyboard binding configurations.
#[derive(Debug, Serialize, Deserialize)]
pub struct Keyboard {
    pub move_right: KeyCode,
    pub move_up: KeyCode,
    pub move_left: KeyCode,
    pub move_down: KeyCode,
    pub dash: KeyCode,
}
impl Default for Keyboard {
    fn default() -> Self {
        Self {
            move_right: KeyCode::ArrowRight,
            move_up: KeyCode::ArrowUp,
            move_left: KeyCode::ArrowLeft,
            move_down: KeyCode::ArrowDown,
            dash: KeyCode::KeyX,
        }
    }
}

#[derive(Resource, Debug, Default, Serialize, Deserialize)]
pub struct Save {
    pub keyboard: Keyboard,
    pub appearance: Appearance,
}

fn read_save(mut commands: Commands) -> Result<()> {
    let save: Save = match std::fs::read_to_string("user.ron") {
        Ok(content) => ron::from_str(&content)?,
        Err(_err) => {
            warn!("Unable to read \"user.ron\", creating default save...");
            Default::default()
        }
    };
    commands.insert_resource(save);
    Ok(())
}

fn write_save_on_exit(mut reader: MessageReader<AppExit>, save: Res<Save>) -> Result<()> {
    if reader.read().next().is_some() {
        let content = ron::to_string(save.as_ref())?;
        std::fs::write("user.ron", content)?;
    }
    Ok(())
}
