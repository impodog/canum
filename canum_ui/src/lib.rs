mod health;
mod prelude;
mod setup;

pub use setup::{BottomCenter, BottomLeft, BottomRight, Center, TopLeft, TopRight};

pub mod text;
pub use text::Fonts;

pub mod image;
pub use image::{Animation, AnimationComplete};

pub mod bar;
pub mod equip;
pub mod lobby;
pub mod shop;

use prelude::*;

pub struct CanumUiPlugin;

impl Plugin for CanumUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            setup::SetupPlugin,
            image::SpritePlugin,
            health::HealthPlugin,
            text::TextPlugin,
            bar::BarPlugin,
            lobby::LobbyPlugin,
            equip::EquipPlugin,
            shop::ShopPlugin,
        ));
    }
}
