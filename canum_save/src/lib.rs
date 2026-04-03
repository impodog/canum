//! Loads and handles user save profiles.

use bevy::prelude::*;
pub(crate) use serde::{Deserialize, Serialize};

mod appearance;
mod controls;
mod progress;

pub use appearance::Lang;

pub struct CanumSavePlugin;
impl Plugin for CanumSavePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, read_save);
        app.add_systems(Startup, init_lang);
        app.add_systems(Last, write_save_on_exit);
    }
}

#[derive(Resource, Debug, Default, Serialize, Deserialize)]
pub struct Save {
    pub keyboard: controls::Keyboard,
    pub appearance: appearance::Appearance,
    pub progress: progress::Progress,
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

pub(crate) fn init_lang(mut commands: Commands, save: Res<Save>) {
    let languages = canum_res::config::CONFIG
        .assets
        .text
        .get(&save.appearance.language)
        .cloned()
        .unwrap_or_else(|| {
            canum_res::config::CONFIG
                .assets
                .text
                .get("en")
                .cloned()
                .unwrap_or_default()
        });
    commands.insert_resource(appearance::Lang(languages.0));
}
