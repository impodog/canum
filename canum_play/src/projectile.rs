use std::sync::Mutex;

use crate::prelude::*;

pub(super) struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DisposeQueue>()
            .init_resource::<ProjectileBounds>();
        app.add_systems(FixedLast, (remove_out_of_bound, dispose_after_collision));
    }
}

/// Defines the boundary for projectiles not to be erased.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct ProjectileBounds(pub Rect);

/// Disables collision with boundaries for certain objects.
#[derive(Component, Default)]
pub struct NoCollideBoundary;

/// Removes itself when out of bounds.
#[derive(Component)]
#[require(Transform)]
pub struct RemoveOutOfBounds {
    /// Scales its distance to the bound center before judging whether to remove.
    pub distance_scale: f32,
}
impl Default for RemoveOutOfBounds {
    fn default() -> Self {
        Self {
            distance_scale: 1.0,
        }
    }
}

/// Marks a projectile either by player or enemy.
#[derive(Component, Debug, Default)]
#[require(
    crate::SessionOnly,
    RemoveOutOfBounds,
    Transform,
    Visibility,
    RigidBody::Dynamic,
    Collider,
    NoCollideBoundary,
    crate::health::Friendly
)]
pub struct Projectile {
    pub no_dispose: bool,
}
impl Projectile {
    pub fn no_dispose(mut self) -> Self {
        self.no_dispose = true;
        self
    }
}

fn remove_out_of_bound(
    commands: ParallelCommands,
    q_projectile: Query<(Entity, &GlobalTransform, &RemoveOutOfBounds)>,
    bounds: Res<ProjectileBounds>,
) {
    let center = bounds.center();
    q_projectile
        .par_iter()
        .for_each(|(entity, transform, removal)| {
            let position = transform.translation().xy();
            let position = (position - center) * removal.distance_scale + center;
            if !bounds.contains(position) {
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
    q_projectile: Query<(Entity, &Projectile)>,
) {
    for entity in queue.drain(..) {
        if let Ok(mut commands) = commands.get_entity(entity) {
            commands.despawn();
        }
    }
    let queue = Mutex::new(queue);
    q_projectile.par_iter().for_each(|(entity, projectile)| {
        if !projectile.no_dispose && collisions.collisions_with(entity).next().is_some() {
            queue.lock().unwrap().push(entity);
        }
    });
}
