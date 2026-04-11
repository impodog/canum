pub mod ant;
pub mod apple;
pub mod lobby;
pub mod turf;

use canum_play::prelude::*;

pub struct CanumS1Plugin;

impl Plugin for CanumS1Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            apple::ApplePlugin,
            lobby::LobbyPlugin,
            turf::TurfPlugin,
            ant::AntPlugin,
        ));
    }
}
