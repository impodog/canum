mod behaviors;
mod defeat;
mod entry;

use crate::*;

static LASER_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Laser".to_owned()));

pub(super) struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            behaviors::BehaviorsPlugin,
            entry::EntryPlugin,
            defeat::DefeatPlugin,
        ));
    }
}

/// Main marker for the apple boss.
#[derive(Component, Default)]
#[require(
    Animation::new("Laser_Static", Vec2::new(64.0, 64.0)),
    Transform::from_translation(Vec3::new(0.0, 0.0, 14.37)),
    RigidBody::Dynamic,
    Collider::rectangle(50.0, 20.0),
    Mass(6.0),
    LockedAxes::TRANSLATION_LOCKED,
    Restitution::new(0.2),
    health::Friendly(false),
    health::ContactDamage { value: 120, projectile: false, order: consts::order::ENEMY_BOSS },
    movements::SpeedDecay(0.75),
    movements::AngularSpeedDecay,
    enemy::health::EnemyHealth::new(4000),
    player::victory::DefeatToWin::default(),
)]
pub struct LaserBoss;

fn laser_friction(q_laser: Query<&mut AngularVelocity, With<LaserBoss>>) {}
