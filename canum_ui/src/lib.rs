mod health;
mod prelude;
mod setup;

pub mod image;
pub use image::{Animation, AnimationComplete};

pub mod transition;

use prelude::*;

pub struct CanumUiPlugin;

impl Plugin for CanumUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            setup::SetupPlugin,
            image::SpritePlugin,
            health::HealthPlugin,
            transition::TransitionPlugin,
        ));
    }
}
