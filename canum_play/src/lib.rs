pub mod controls;
pub mod general;
pub mod player;
mod prelude;

use bevy::prelude::*;

pub struct CanumPlayPlugin;

impl Plugin for CanumPlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            player::PlayerPlugin,
            controls::ControlsPlugin,
            general::GeneralPlugin,
        ));
    }
}
