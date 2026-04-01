use std::sync::Mutex;

use crate::prelude::*;

pub(super) struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DisposeQueue>();
        app.add_systems(FixedPostUpdate, remove_out_of_bound_projectiles);
        app.add_systems(FixedLast, dispose_after_collision);
    }
}

/// Deisables collision with boundaries for certain objects.
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
    q_projectile: Query<(Entity, &Transform), With<Projectile>>,
) {
    let virtual_size = (
        CONFIG.display.virtual_size.0 as f32,
        CONFIG.display.virtual_size.1 as f32,
    );
    q_projectile.par_iter().for_each(|(entity, transform)| {
        if transform.translation.x.abs() > virtual_size.0
            || transform.translation.y.abs() > virtual_size.1
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
