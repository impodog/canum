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

/// User keyboard binding configurations.
#[derive(Resource, Debug, Serialize, Deserialize)]
pub struct Keyboard {
    pub move_right: KeyCode,
    pub move_up: KeyCode,
    pub move_left: KeyCode,
    pub move_down: KeyCode,
}
impl Default for Keyboard {
    fn default() -> Self {
        Self {
            move_right: KeyCode::KeyD,
            move_up: KeyCode::KeyW,
            move_left: KeyCode::KeyA,
            move_down: KeyCode::KeyS,
        }
    }
}

#[derive(Resource, Debug, Default, Serialize, Deserialize)]
pub struct Save {
    pub keyboard: Keyboard,
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
