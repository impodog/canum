mod lobby;

use canum_play::prelude::*;

pub struct CanumS3Plugin;

impl Plugin for CanumS3Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((lobby::LobbyPlugin,));
    }
}
