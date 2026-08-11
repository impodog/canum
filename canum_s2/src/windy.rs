mod entry;
mod obstacles;
mod wind;

use crate::prelude::*;

pub static WINDY_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Windy".to_owned()));

pub(super) struct WindyPlugin;

impl Plugin for WindyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            entry::EntryPlugin,
            wind::WindPlugin,
            obstacles::ObstaclesPlugin,
        ));
    }
}

#[derive(Component, Default)]
#[require(health::Friendly(false), player::victory::DefeatToWin)]
pub struct WindyMainEntity;
