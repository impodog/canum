use crate::util::DespawnBehavior;
use bevy::prelude::*;
use bevy::transform::TransformSystems;

pub(super) struct TransformPlugin;

impl Plugin for TransformPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (spawn_parent, update_parent)
                .chain()
                .before(TransformSystems::Propagate),
        );
    }
}

/// Marks this entity as rigidly following `target` in world space.
/// This must be applied to a entity without a real parent.
///
/// # Note
/// This does not work for nested follows.
#[derive(Component, Debug)]
#[non_exhaustive]
pub struct Follow {
    pub target: Entity,
    pub despawn_behavior: DespawnBehavior,
}
impl Follow {
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            despawn_behavior: default(),
        }
    }
    pub fn with_despawn_behavior(mut self, despawn_behavior: DespawnBehavior) -> Self {
        self.despawn_behavior = despawn_behavior;
        self
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
struct DummyParent {
    copy_from: Entity,
}

/// Runs in PostUpdate, before Bevy's transform propagation.
pub fn spawn_parent(
    mut q_follow: Query<(Entity, &Follow, Option<&ChildOf>)>,
    q_exists: Query<()>,
    commands: ParallelCommands,
) {
    q_follow
        .par_iter_mut()
        .for_each(|(entity, follow, parent)| {
            if q_exists.get(follow.target).is_err() {
                match follow.despawn_behavior {
                    DespawnBehavior::Despawn => {
                        commands.command_scope(|mut commands| {
                            if let Some(parent) = parent {
                                commands.entity(parent.0).despawn();
                            } else {
                                commands.entity(entity).despawn();
                            }
                        });
                    }
                    DespawnBehavior::Remove => {
                        commands.command_scope(|mut commands| {
                            commands
                                .entity(entity)
                                .remove::<Follow>()
                                .remove::<ChildOf>();
                            if let Some(parent) = parent {
                                commands.entity(parent.0).despawn();
                            }
                        });
                    }
                }
            } else if parent.is_none() {
                commands.command_scope(|mut commands| {
                    let parent = commands
                        .spawn(DummyParent {
                            copy_from: follow.target,
                        })
                        .id();
                    commands.entity(entity).insert(ChildOf(parent));
                });
            }
        });
}

fn update_parent(
    mut q_parent: Query<(&mut Transform, &DummyParent)>,
    q_transform: Query<&GlobalTransform>,
) {
    q_parent.par_iter_mut().for_each(|(mut transform, dummy)| {
        let Ok(global_transform) = q_transform.get(dummy.copy_from) else {
            return;
        };
        *transform = global_transform.compute_transform();
    });
}
