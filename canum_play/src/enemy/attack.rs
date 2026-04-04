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
