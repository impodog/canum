use std::sync::LazyLock;

use canum_play::prelude::*;

mod background;

pub(super) struct ApplePlugin;

static APPLE_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Apple".to_owned()));

impl Plugin for ApplePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((background::BackgroundPlugin,));
        app.add_observer(spawn_apple);
    }
}

/// Main marker for the apple boss.
#[derive(Component, Default)]
#[require(
    Animation::new("AppleStatic", Vec2::new(64.0, 64.0)),
    Transform::from_translation(Vec3::new(-10.0, 150.0, 14.37)),
    RigidBody::Dynamic,
    Collider::circle(20.0),
    Mass(30.0),
    LockedAxes::ROTATION_LOCKED,
    Restitution::new(0.5),
    health::Friendly(false),
    health::ContactDamage { value: 100, projectile: false, order: 100 },
    enemy::health::EnemyHealth::new(7000),
    enemy::health::DamageSound::new("Apple_Damage"),
)]
pub struct AppleBoss;

fn spawn_apple(_event: On<background::AppleTreeBackgroundChanged>, mut commands: Commands) {
    commands.spawn((AppleBoss,));
}
