pub mod prelude;

pub mod consts;
pub mod controls;
pub mod enemy;
pub mod health;
pub mod math;
pub mod movements;
pub mod player;
pub mod projectile;
pub mod setup;

use bevy::prelude::*;

pub use setup::SessionOnly;

pub struct CanumPlayPlugin;

impl Plugin for CanumPlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            player::PlayerPlugin,
            controls::ControlsPlugin,
            movements::MovementsPlugin,
            health::HealthPlugin,
            setup::SetupPlugin,
            enemy::EnemyPlugin,
            projectile::ProjectilePlugin,
        ));
    }
}
