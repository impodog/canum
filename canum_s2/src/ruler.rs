mod behaviors;
mod entry;

use super::*;

pub static RULER_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Ruler".to_owned()));
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct RulerSet;

pub(super) struct RulerPlugin;

impl Plugin for RulerPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(FixedUpdate, RulerSet.run_if(in_state(RULER_STATE.clone())));
        app.add_plugins((entry::EntryPlugin, behaviors::BehaviorsPlugin));
    }
}

pub const LENGTH_RATIO: f32 = 100.0 / 15.0;
pub const INITIAL_HEIGHT: f32 = 15.0;
pub const INITIAL_HITBOX_HEIGHT: f32 = 14.0;
pub const HITBOX_RATIO: f32 = INITIAL_HITBOX_HEIGHT / INITIAL_HEIGHT;
pub const SIZE: Vec2 = vec2(LENGTH_RATIO * INITIAL_HEIGHT, INITIAL_HEIGHT);

#[derive(Component)]
#[require(
    Animation::new("Ruler_Ruler", SIZE),
    RigidBody::Kinematic,
    Collider::rectangle(LENGTH_RATIO * INITIAL_HITBOX_HEIGHT, INITIAL_HITBOX_HEIGHT),
    CollidingEntities,
    Mass(20.0),
    Restitution::new(0.35),
    health::Friendly(false),
    health::ContactDamage { value: 130, projectile: false, order: consts::order::ENEMY_BOSS },
    movements::SpeedDecay(0.8),
    enemy::health::EnemyHealth::new(3000),
    enemy::health::DamageSound::new("Wcat_Damage"),
    projectile::NoCollideBoundary,
)]
pub struct RulerBoss;
