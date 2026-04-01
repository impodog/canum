use crate::prelude::*;

pub mod boss;
pub mod health;
pub mod movements;

pub(super) struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((movements::MovementsPlugin, health::HealthPlugin));
    }
}
