//! This module loads resources and display them accordingly.

pub mod config;

mod sprite;
pub use sprite::*;

mod framerate;

pub mod camera;
pub mod window;

use bevy::prelude::*;

pub struct CanumResPlugin;

impl Plugin for CanumResPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnimationAtlasHandles>()
            .init_resource::<camera::VirtualResolution>()
            .init_resource::<window::WindowTitle>();
        app.add_systems(Update, (modify_animation, tick_animation));
        app.add_systems(Last, framerate::control_framerate);
        app.add_systems(Startup, (camera::setup_camera, window::setup_window));
        app.add_systems(Update, (window::update_window, camera::update_camera));
    }
}
