mod health;
mod prelude;
mod setup;

pub mod sprite;
pub use sprite::{Animation, AnimationComplete};

use prelude::*;

pub struct CanumUiPlugin;

impl Plugin for CanumUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            setup::SetupPlugin,
            sprite::SpritePlugin,
            health::HealthPlugin,
        ));
    }
}
