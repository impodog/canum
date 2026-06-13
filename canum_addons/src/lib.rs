mod achievements;
mod charms;
mod weapons;

use canum_play::prelude::*;

pub struct CanumAddonsPlugin;

impl Plugin for CanumAddonsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            charms::CharmsPlugin,
            weapons::WeaponsPlugin,
            achievements::AchievementsPlugin,
        ));
    }
}
