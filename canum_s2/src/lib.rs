pub mod bread;
pub mod laser;
pub mod lobby;
pub mod projected;
pub mod ruler;
pub mod windy;

use canum_play::prelude::*;

pub struct CanumS2Plugin;

impl Plugin for CanumS2Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            lobby::LobbyPlugin,
            laser::LaserPlugin,
            windy::WindyPlugin,
            bread::BreadPlugin,
            projected::ProjectedPlugin,
            ruler::RulerPlugin,
        ));
    }
}
