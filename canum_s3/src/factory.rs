mod entry;
mod robot;

use canum_play::prelude::*;

pub(super) struct FactoryPlugin;

pub static FACTORY_STATE: LazyLock<setup::Fight> =
    LazyLock::new(|| setup::Fight("Factory".to_owned()));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct FactorySystems;

impl Plugin for FactoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((entry::EntryPlugin, robot::BehaviorsPlugin));
        app.configure_sets(
            FixedUpdate,
            FactorySystems.run_if(in_state(FACTORY_STATE.clone())),
        );
    }
}

#[derive(Component, Default)]
#[require(
    Animation::new("Factory_Robot", Vec2::new(64.0, 64.0)),
    Transform::from_translation(Vec3::new(0.0, 0.0, 14.37)),
    RigidBody::Dynamic,
    Collider::rectangle(32.0, 32.0),
    Mass(9.0),
    LockedAxes::ROTATION_LOCKED,
    Restitution::new(0.6),
    health::Friendly(false),
    health::ContactDamage { value: consts::damage::ONE_MID, projectile: false, order: consts::order::ENEMY_BOSS },
    movements::SpeedDecay(0.5),
    enemy::health::EnemyHealth::new(3200),
    enemy::health::DamageSound::new("Wcat_Damage"),
    canum_fx::emphasis::LeaveTrail::new(0.1, 0.12),
)]
pub struct RobotBoss;
