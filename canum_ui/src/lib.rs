mod health;
mod menu;
mod prelude;
mod setup;
mod text;

pub mod image;
pub use image::{Animation, AnimationComplete};

use prelude::*;

pub struct CanumUiPlugin;

impl Plugin for CanumUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            setup::SetupPlugin,
            image::SpritePlugin,
            health::HealthPlugin,
            menu::MenuPlugin,
            text::TextPlugin,
        ));
    }
}
