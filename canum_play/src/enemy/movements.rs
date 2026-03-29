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
                world.commands().entity(entity).insert(DisplacementInfo {
                    start_time,
                    prev_velocity: Default::default(),
                });
            });
        app.add_systems(FixedUpdate, (work_displacement,));
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct Displacement {
    pub displace: Vec2,
    pub duration: Duration,
}
#[derive(Component, Debug, Clone, Default)]
struct DisplacementInfo {
    start_time: Duration,
    prev_velocity: Vec2,
}

fn work_displacement(
    commands: ParallelCommands,
    mut q_displacement: Query<(
        Entity,
        &mut LinearVelocity,
        &Displacement,
        &mut DisplacementInfo,
    )>,
    time: Res<Time>,
) {
    fn derivative(value: f32) -> f32 {
        (CubicInOutCurve.sample(value + 1e-6).unwrap() - CubicInOutCurve.sample(value).unwrap())
            / 1e-6
    }
    q_displacement.par_iter_mut().for_each(
        |(entity, mut linear_velocity, displacement, mut info)| {
            let ratio = (time.elapsed() - info.start_time).as_secs_f32()
                / displacement.duration.as_secs_f32();
            if ratio >= 1.0 {
                **linear_velocity -= info.prev_velocity;
                commands.command_scope(|mut commands| {
                    commands
                        .entity(entity)
                        .remove::<Displacement>()
                        .remove::<DisplacementInfo>();
                });
                return;
            }
            let new_velocity =
                derivative(ratio) / displacement.duration.as_secs_f32() * displacement.displace;
            **linear_velocity += new_velocity - info.prev_velocity;
            info.prev_velocity = new_velocity;
        },
    );
}
