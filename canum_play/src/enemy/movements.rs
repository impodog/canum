use crate::movements::*;
use crate::prelude::*;

use bevy::ecs::lifecycle::HookContext;
use std::time::Duration;

pub(super) struct MovementsPlugin;

impl Plugin for MovementsPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<Displacement>()
            .on_add(|mut world, HookContext { entity, .. }| {
                let start_time = world.get_resource::<Time>().unwrap().elapsed();
                let partial_velocity = world
                    .commands()
                    .spawn((ChildOf(entity), PartialVelocity::unlinked()))
                    .id();
                world.commands().entity(entity).insert(DisplacementInfo {
                    start_time,
                    partial_velocity,
                });
            });
        app.add_systems(FixedUpdate, (work_displacement,));
    }
}

/// Calls for the entity to move in quadratic speed changed with a fixed displacement.
/// This is directly added to the entity, to enhance efficiency.
#[derive(Component, Debug, Clone, Default)]
#[require(ForcedVelocity)]
pub struct Displacement {
    pub displace: Vec2,
    pub duration: Duration,
    pub notify: Option<Entity>,
}
#[derive(Component, Debug, Clone)]
struct DisplacementInfo {
    start_time: Duration,
    partial_velocity: Entity,
}
/// This will be sent to `notify` Entity, if any.
#[derive(EntityEvent)]
pub struct DisplacementComplete {
    pub entity: Entity,
}

fn work_displacement(
    commands: ParallelCommands,
    q_displacement: Query<(Entity, &Displacement, &DisplacementInfo)>,
    q_partial_velocity: Query<Mut<PartialVelocity>>,
    time: Res<Time>,
) {
    fn derivative(value: f32) -> f32 {
        (QuadraticOutCurve.sample(value + 1e-6).unwrap() - QuadraticOutCurve.sample(value).unwrap())
            / 1e-6
    }
    let q_partial_velocity = std::sync::Mutex::new(q_partial_velocity);
    q_displacement
        .par_iter()
        .for_each(|(entity, displacement, info)| {
            let ratio = (time.elapsed() - info.start_time).as_secs_f32()
                / displacement.duration.as_secs_f32();
            if ratio >= 1.0 {
                commands.command_scope(|mut commands| {
                    commands
                        .entity(entity)
                        .remove::<Displacement>()
                        .remove::<DisplacementInfo>();
                    if let Some(notify_entity) = displacement.notify {
                        commands.trigger(DisplacementComplete {
                            entity: notify_entity,
                        });
                    }
                });
                return;
            }
            let new_velocity =
                derivative(ratio) / displacement.duration.as_secs_f32() * displacement.displace;
            {
                let mut q_partial_velocity = q_partial_velocity.lock().unwrap();
                let Ok(mut partial_velocity) = q_partial_velocity.get_mut(info.partial_velocity)
                else {
                    return;
                };
                **partial_velocity = new_velocity;
            }
        });
}
