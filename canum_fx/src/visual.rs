use crate::prelude::*;
use bevy::ecs::lifecycle::HookContext;

pub(super) struct VisualPlugin;

impl Plugin for VisualPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<Shadow>()
            .on_insert(|mut world, HookContext { entity, .. }| {
                let Some(shadow) = world.get::<Shadow>(entity) else {
                    return;
                };
                let length = shadow.0;
                let Some(mut animation) = world.get_mut::<canum_res::Animation>(entity) else {
                    return;
                };
                *animation = canum_res::Animation::new("Shadow", vec2(length, length * 0.5));
                let Some(mut transform) = world.get_mut::<Transform>(entity) else {
                    return;
                };
                transform.translation.y = -length * 0.5;
            });

        app.add_systems(PreUpdate, reset_transform)
            // Just before the end of the frame, apply the shake.
            // This is ordered so that the transform propagation produces correct values for the global transform, which is used by Bevy's rendering.
            .add_systems(PostUpdate, shake_camera.before(TransformSystems::Propagate))
            .add_systems(PostStartup, setup_camera)
            .add_observer(respond_shake_camera);
    }
}

/// A shadow of given length.
#[derive(Component, Debug, Clone)]
#[require(canum_res::Animation, Transform::from_translation(vec3(0.0, 0.0, -0.1)))]
pub struct Shadow(pub f32);
impl Default for Shadow {
    fn default() -> Self {
        Self(32.0)
    }
}

use bevy::math::ops::powf;

const TRAUMA_DECAY_PER_SECOND: f32 = 0.5;
const TRAUMA_EXPONENT: f32 = 2.0;
const MAX_ANGLE: f32 = 10.0_f32.to_radians();
const MAX_TRANSLATION: f32 = 20.0;
const NOISE_SPEED: f32 = 18.0;

/// The current state of the camera shake that is updated every frame.
#[derive(Component, Debug, Default)]
struct CameraShakeState {
    trauma: f32,
    /// The original transform of the camera before applying the shake.
    /// We store this so that we can restore the camera's transform to its original state at the start of the next frame.
    original_transform: Transform,
}

#[derive(Component, Debug)]
#[require(CameraShakeState)]
struct CameraShakeConfig {
    trauma_decay_per_second: f32,
    exponent: f32,
    max_angle: f32,
    max_translation: f32,
    noise_speed: f32,
}

/// Shakes the camera by a given trauma value.
#[derive(Event, Clone, Copy, Debug)]
pub struct ShakeCamera(pub f32);

fn respond_shake_camera(event: On<ShakeCamera>, camera_shake: Single<&mut CameraShakeState>) {
    let mut camera_shake = camera_shake.into_inner();
    camera_shake.trauma = (camera_shake.trauma + event.0).clamp(0.0, 1.0);
}

/// Let's start with the core mechanic: how do we shake the camera?
/// This system runs right at the end of the frame, so that we can sneak in the shake effect before rendering kicks in.
fn shake_camera(
    camera_shake: Single<(&mut CameraShakeState, &CameraShakeConfig, &mut Transform)>,
    time: Res<Time>,
) {
    let (mut camera_shake, config, mut transform) = camera_shake.into_inner();

    camera_shake.original_transform = *transform;

    let t = time.elapsed_secs() * config.noise_speed;

    let rotation_noise = perlin_noise::generate(t + 0.0);
    let x_noise = perlin_noise::generate(t + 143.7);
    let y_noise = perlin_noise::generate(t + 243.7);

    let shake = powf(camera_shake.trauma, config.exponent);

    let roll_offset = rotation_noise * shake * config.max_angle;
    let x_offset = x_noise * shake * config.max_translation;
    let y_offset = y_noise * shake * config.max_translation;

    transform.translation.x += x_offset;
    transform.translation.y += y_offset;
    transform.rotate_z(roll_offset);

    camera_shake.trauma -= config.trauma_decay_per_second * time.delta_secs();
    camera_shake.trauma = camera_shake.trauma.clamp(0.0, 1.0);
}

fn reset_transform(camera_shake: Single<(&CameraShakeState, &mut Transform)>) {
    let (camera_shake, mut transform) = camera_shake.into_inner();
    *transform = camera_shake.original_transform;
}

fn setup_camera(
    mut commands: Commands,
    q_camera: Query<Entity, With<canum_res::camera::MainCamera>>,
) {
    let Ok(camera) = q_camera.single() else {
        return;
    };
    commands.entity(camera).insert(CameraShakeConfig {
        trauma_decay_per_second: TRAUMA_DECAY_PER_SECOND,
        exponent: TRAUMA_EXPONENT,
        max_angle: MAX_ANGLE,
        max_translation: MAX_TRANSLATION,
        noise_speed: NOISE_SPEED,
    });
}

/// Tiny 1D Perlin noise implementation. The mathematical details are not important here.
mod perlin_noise {
    use super::*;

    pub fn generate(x: f32) -> f32 {
        // Left coordinate of the unit-line that contains the input.
        let x_floor = x.floor() as usize;

        // Input location in the unit-line.
        let xf0 = x - x_floor as f32;
        let xf1 = xf0 - 1.0;

        // Wrap to range 0-255.
        let xi0 = x_floor & 0xFF;
        let xi1 = (x_floor + 1) & 0xFF;

        // Apply the fade function to the location.
        let t = fade(xf0).clamp(0.0, 1.0);

        // Generate hash values for each point of the unit-line.
        let h0 = PERMUTATION_TABLE[xi0];
        let h1 = PERMUTATION_TABLE[xi1];

        // Linearly interpolate between dot products of each gradient with its distance to the input location.
        let a = dot_grad(h0, xf0);
        let b = dot_grad(h1, xf1);
        a.interpolate_stable(&b, t)
    }

    // A cubic curve that smoothly transitions from 0 to 1 as t goes from 0 to 1
    fn fade(t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    fn dot_grad(hash: u8, xf: f32) -> f32 {
        // In 1D case, the gradient may be either 1 or -1.
        // The distance vector is the input offset (relative to the smallest bound).
        if hash & 0x1 != 0 { xf } else { -xf }
    }

    // Perlin noise permutation table. This is a random sequence of the numbers 0-255.
    const PERMUTATION_TABLE: [u8; 256] = [
        0x7a, 0xc0, 0x72, 0xee, 0xd, 0x17, 0xf0, 0x55, 0x35, 0xcb, 0xe4, 0xf7, 0x3f, 0xc3, 0x7,
        0x56, 0xbb, 0xfa, 0x83, 0x21, 0xb3, 0x12, 0x44, 0xd1, 0xd8, 0x22, 0xc8, 0xd2, 0xac, 0xa4,
        0x54, 0x1a, 0xa1, 0xa2, 0x38, 0x5b, 0x50, 0xb5, 0x43, 0x15, 0x8, 0x4d, 0x9d, 0xa0, 0xa7,
        0x8e, 0x51, 0x59, 0xcf, 0x37, 0xbc, 0x8c, 0x2, 0x98, 0xd6, 0xf8, 0x5a, 0xa5, 0xe8, 0x11,
        0xef, 0x78, 0x41, 0xf2, 0x5d, 0x10, 0xdc, 0x77, 0x80, 0x0, 0xe0, 0x36, 0xcd, 0x48, 0x13,
        0x2f, 0x1c, 0x96, 0xb6, 0xb0, 0xea, 0x3, 0xc, 0x49, 0x66, 0x65, 0x31, 0x30, 0x91, 0x73,
        0x19, 0x6d, 0xe1, 0xd9, 0xe6, 0xa6, 0xc9, 0xec, 0x4, 0x46, 0x2c, 0xf3, 0x3b, 0x6c, 0xc7,
        0x2b, 0x6, 0xd7, 0x2a, 0xca, 0x90, 0xfe, 0x1d, 0x2d, 0x64, 0x6e, 0x42, 0xdd, 0xb1, 0x4a,
        0x76, 0x9c, 0x67, 0xfd, 0x5c, 0x70, 0x29, 0x40, 0xe9, 0xe7, 0x1b, 0xed, 0x9, 0xf4, 0xdb,
        0xc5, 0xbd, 0x85, 0xb2, 0x1, 0x28, 0x88, 0x86, 0xbe, 0x93, 0xeb, 0xa, 0xc4, 0x3e, 0xf1,
        0x47, 0x62, 0x74, 0xcc, 0x9b, 0xda, 0xc2, 0x45, 0x87, 0x9f, 0x3d, 0x6f, 0x81, 0x26, 0xad,
        0x79, 0xde, 0xa3, 0x1e, 0xe2, 0x95, 0x7e, 0x52, 0xd3, 0x9e, 0x94, 0x5, 0xf9, 0xd4, 0xe3,
        0x16, 0x6b, 0xaa, 0xf6, 0x68, 0xb9, 0x97, 0xbf, 0x24, 0x69, 0x32, 0x33, 0xf, 0x7d, 0x1f,
        0x25, 0x82, 0xfb, 0xf5, 0x4e, 0x57, 0x8f, 0xb4, 0x8d, 0x71, 0x27, 0xa9, 0xe5, 0x7b, 0xc1,
        0x23, 0x53, 0x4b, 0x8a, 0xb, 0x20, 0xdf, 0x7c, 0x63, 0xae, 0x2e, 0xce, 0xb7, 0x60, 0x7f,
        0x4f, 0x9a, 0x6a, 0xa8, 0x34, 0x99, 0x89, 0x58, 0xaf, 0xc6, 0xb8, 0x18, 0x84, 0x3c, 0x39,
        0x92, 0xd0, 0xfc, 0x3a, 0x5f, 0x4c, 0x14, 0xd5, 0x5e, 0xff, 0xab, 0xba, 0xe, 0x61, 0x8b,
        0x75,
    ];
}
