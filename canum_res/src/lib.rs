//! This module loads resources and display them accordingly.

pub mod config;

mod sprite;
pub use sprite::*;

mod font;
pub use font::*;

mod framerate;

pub mod background;
pub mod camera;
pub mod sound;
pub mod window;

use bevy::prelude::*;

pub struct CanumResPlugin;

impl Plugin for CanumResPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnimationAtlasHandles>()
            .init_resource::<AnimationImageHandles>()
            .init_resource::<UpdateHandleStrongQueue>()
            .init_resource::<UpdateTimes>()
            .init_resource::<camera::VirtualResolution>()
            .init_resource::<window::WindowTitle>()
            .init_resource::<sound::LoadedSounds>();
        app.add_systems(First, (tick_animation, modify_animation).chain());
        app.add_systems(
            Last,
            (
                sprite::update_handle_strong,
                sprite::random_clearing,
                framerate::control_framerate,
            )
                .chain(),
        );
        app.add_systems(
            Startup,
            (camera::setup_camera, window::setup_window, font::setup_font),
        );
        app.add_systems(Update, (window::update_window, camera::update_camera));
        app.add_systems(
            FixedUpdate,
            (
                sound::fade_in,
                sound::fade_out,
                sound::start_playing_sound,
                sound::change_music,
                sound::update_sound,
            ),
        );
        app.add_observer(sprite::update_handle);
        app.add_plugins(background::BackgroundPlugin);
    }
}
