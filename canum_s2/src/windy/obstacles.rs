use super::*;

pub(super) struct ObstaclesPlugin;

impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().register_component_hooks::<Spike>().on_add(
            |mut world, HookContext { entity, .. }| {
                let Some(spike) = world.get::<Spike>(entity) else {
                    return;
                };
                let length = spike.0;
                let half_length = length * 0.5;
                world.commands().entity(entity).insert((
                    Animation::new("Windy_Spike", vec2(length, length)),
                    Collider::triangle(
                        vec2(-half_length, -half_length),
                        vec2(half_length, -half_length),
                        vec2(0.0, half_length),
                    ),
                ));
            },
        );
    }
}

#[derive(Component, Default)]
#[require(
    RigidBody::Kinematic,
    Collider,
    health::Friendly(false),
    health::ContactDamage { value: 50, projectile: false, order: consts::order::ENEMY_PROJ },
    projectile::NoCollideBoundary,
    projectile::RemoveOutOfBounds,
    movements::ForcedVelocity,
    Visibility,
)]
struct Obstacle;

/// A spike of given side length.
#[derive(Component, Default)]
#[require(Obstacle, obstacle::BumpAway {direction: Dir2::from_xy_unchecked(0.0, 1.0), strength: 30.0})]
pub struct Spike(pub f32);
