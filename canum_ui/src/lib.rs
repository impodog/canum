mod health;
mod prelude;
mod setup;

pub use setup::{BottomCenter, BottomLeft, Center, TopLeft};

pub mod text;
pub use text::Fonts;

pub mod image;
pub use image::{Animation, AnimationComplete};

pub mod boss;

pub mod bar;

use prelude::*;

pub struct CanumUiPlugin;

impl Plugin for CanumUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            setup::SetupPlugin,
            image::SpritePlugin,
            health::HealthPlugin,
            text::TextPlugin,
            boss::BossPlugin,
            bar::BarPlugin,
        ));
    }
}
