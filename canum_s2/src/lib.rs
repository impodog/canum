pub mod lobby;

use canum_play::prelude::*;

pub struct CanumS2Plugin;

impl Plugin for CanumS2Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((lobby::LobbyPlugin,));
    }
}
