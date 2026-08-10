use crate::prelude::*;
use bevy::ecs::lifecycle::HookContext;

pub(super) struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<LaserNode>()
            .on_insert(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(laser_destroy);
            });
        app.world_mut()
            .register_component_hooks::<LaserLike>()
            .on_insert(|mut world, HookContext { entity, .. }| {
                let Some(root) = world.get::<LaserLike>(entity) else {
                    return;
                };
                let root = root.clone();
                let spawn = world
                    .commands()
                    .spawn((
                        ChildOf(entity),
                        LaserNode {
                            prev: entity,
                            next: None,
                        },
                        root.collider,
                        root.terminal,
                    ))
                    .id();
                world.commands().trigger(LaserSpawn { entity, spawn });
            });
        app.add_systems(FixedPreUpdate, laser_work);
    }
}

/// Creates laser like shooting effects and sends touch events.
/// This shoots to the right if not rotated.
///
/// You can update laser node with each `LaserSpawn` event.
/// You can apply effects(that is to say, damage is not managed here) with each `LaserTouch` event.
/// Both events are sent to the central entity.
///
/// ### Warning
/// The first node's event is trigger right after inserting `LaserLike`. So if you want to observe that make sure to spawn the observer BEFORE adding `LaserLike`.
#[derive(Component, Debug, Clone, Default)]
#[require(Transform, Visibility, LaserMarker)]
pub struct LaserLike {
    /// In the middle of the laser beam.
    pub middle: canum_res::Animation,
    /// At the end of the beam.
    pub terminal: canum_res::Animation,
    pub collider: Collider,
    /// Length of each laser node.
    pub length: f32,
}

#[derive(EntityEvent, Debug)]
pub struct LaserSpawn {
    /// This is the parent `LaserLike` entity.
    pub entity: Entity,
    /// The new spawn laser node entity.
    pub spawn: Entity,
}

#[derive(EntityEvent, Debug)]
pub struct LaserTouch {
    /// This is the parent `LaserLike` entity.
    pub entity: Entity,
    /// The end entity that the laser touches.
    pub target: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
#[require(canum_res::Animation, RigidBody::Kinematic, Sensor, LaserMarker)]
struct LaserNode {
    prev: Entity,
    next: Option<Entity>,
}
/// Dummy component for easier querying.
#[derive(Component, Default)]
struct LaserMarker;

#[derive(EntityEvent)]
struct LaserDestroy(Entity);

fn laser_work(
    commands: ParallelCommands,
    collisions: Collisions,
    mut q_laser: Query<(
        Entity,
        &mut LaserNode,
        &mut canum_res::Animation,
        &Transform,
        &ChildOf,
    )>,
    q_root: Query<&LaserLike>,
    q_laser_marker: Query<(), With<LaserMarker>>,
) {
    q_laser
        .par_iter_mut()
        .for_each(|(entity, mut node, mut animation, transform, parent)| {
            let Ok(root) = q_root.get(parent.0) else {
                return;
            };
            if !q_laser_marker.get(node.prev).is_ok() {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).try_despawn();
                });
                return;
            }
            let mut any_colliding = false;
            for other in collisions.entities_colliding_with(entity) {
                if !q_laser_marker.get(other).is_ok() {
                    any_colliding = true;
                    commands.command_scope(|mut commands| {
                        commands.trigger(LaserTouch {
                            entity: parent.0,
                            target: other,
                        });
                    });
                }
            }
            if any_colliding && let Some(next) = node.next {
                commands.command_scope(|mut commands| {
                    commands.trigger(LaserDestroy(next));
                });
                node.next = None;
                *animation = root.terminal.clone();
            }
            if !any_colliding && node.next.is_none() {
                let mut new_transform = *transform;
                new_transform.translation.x += root.length;
                let next_node = commands.command_scope(|mut commands| {
                    let next_node = commands
                        .spawn((
                            ChildOf(parent.0),
                            LaserNode {
                                prev: entity,
                                next: None,
                            },
                            new_transform,
                            root.collider.clone(),
                            root.terminal.clone(),
                        ))
                        .id();
                    commands.trigger(LaserSpawn {
                        entity: parent.0,
                        spawn: next_node,
                    });
                    next_node
                });
                *animation = root.middle.clone();
                node.next = Some(next_node);
            }
        });
}

fn laser_destroy(event: On<LaserDestroy>, mut commands: Commands, q_laser: Query<&LaserNode>) {
    let Ok(node) = q_laser.get(event.0) else {
        return;
    };
    if let Some(next) = node.next {
        commands.trigger(LaserDestroy(next));
    }
    commands.entity(event.0).try_despawn();
}
