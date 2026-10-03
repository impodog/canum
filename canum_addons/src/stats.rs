mod player_stats;

use super::*;

pub(super) struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((player_stats::PlayerStatsPlugin,));
    }
}
