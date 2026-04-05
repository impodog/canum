use crate::prelude::*;

pub(super) struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component, Default)]
#[require(
    crate::projectile::Projectile,
    crate::health::Friendly(false),
    crate::health::ContactDamage {value: 100, projectile: true, order: crate::consts::order::ENEMY_PROJ},
    Animation
)]
pub struct EnemyProjectile;

/// Marks a enemy minion. This doesn't behave like a projectile, which will be disposed when out of bound or contacted.
#[derive(Component, Default)]
#[require(
    crate::SessionOnly,
    Transform,
    RigidBody::Dynamic,
    Collider,
    LockedAxes::ROTATION_LOCKED,
    projectile::NoCollideBoundary,
    crate::health::Friendly(false),
    crate::health::ContactDamage {value: 200, projectile: true, order: crate::consts::order::ENEMY_MINION},
)]
pub struct Minion;
