use crate::prelude::*;

pub(super) struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component, Default)]
#[require(
    crate::projectile::Projectile,
    crate::health::Friendly(false),
    crate::health::ContactDamage {value: 100, projectile: true, order: 50},
    Animation
)]
pub struct EnemyProjectile;
