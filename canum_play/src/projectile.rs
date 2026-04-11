use std::sync::Mutex;

use crate::prelude::*;

pub(super) struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DisposeQueue>()
            .init_resource::<ProjectileBounds>();
        app.add_systems(FixedPostUpdate, remove_out_of_bound_projectiles);
        app.add_systems(FixedLast, dispose_after_collision);
    }
}

/// Defines the boundary for projectiles not to be erased.
#[derive(Resource, Default)]
pub struct ProjectileBounds {
    pub min: Vec2,
    pub max: Vec2,
}

/// Disables collision with boundaries for certain objects.
#[derive(Component, Default)]
pub struct NoCollideBoundary;

/// Marks a projectile either by player or enemy.
#[derive(Component, Debug, Default)]
#[require(
    crate::SessionOnly,
    Transform,
    RigidBody::Dynamic,
    Collider,
    NoCollideBoundary,
    crate::health::Friendly
)]
pub struct Projectile;

fn remove_out_of_bound_projectiles(
    commands: ParallelCommands,
    q_projectile: Query<(Entity, &GlobalTransform), With<Projectile>>,
    bounds: Res<ProjectileBounds>,
) {
    q_projectile.par_iter().for_each(|(entity, transform)| {
        let position = transform.translation().xy();
        if position.x > bounds.max.x
            || position.x < bounds.min.x
            || position.y > bounds.max.y
            || position.y < bounds.min.y
        {
            commands.command_scope(|mut commands| {
                commands.entity(entity).despawn();
            });
        }
    });
}

/// Projectiles are removed one fixed frame after collision detected.
#[derive(Resource, Default, Deref, DerefMut)]
struct DisposeQueue(Vec<Entity>);

fn dispose_after_collision(
    mut commands: Commands,
    mut queue: ResMut<DisposeQueue>,
    collisions: Collisions,
    q_projectile: Query<Entity, With<Projectile>>,
) {
    for entity in queue.drain(..) {
        if let Ok(mut commands) = commands.get_entity(entity) {
            commands.despawn();
        }
    }
    let queue = Mutex::new(queue);
    q_projectile.par_iter().for_each(|entity| {
        if collisions.collisions_with(entity).next().is_some() {
            queue.lock().unwrap().push(entity);
        }
    });
}
