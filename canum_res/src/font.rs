use bevy::prelude::*;
pub use bevy_image_font::{
    ImageFont, ImageFontScalingMode, ImageFontText, LetterSpacing,
    atlas_sprites::ImageFontSpriteText,
    rendered::{ImageFontPreRenderedText, ImageFontPreRenderedUiText},
};

#[derive(Resource, Default, Debug, Clone)]
pub struct PixelFonts {
    pub normal: Handle<ImageFont>,
}

pub(crate) fn setup_font(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(PixelFonts {
        normal: server.load::<ImageFont>("./assets/fonts/normal.image_font.ron"),
    });
}
