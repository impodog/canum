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
        app.add_systems(FixedLast, (update_forced_velocity, speed_decay).chain());
    }
}

/// Marks this entity to decrease speed gradually, for the part minus forced velocity.
#[derive(Component, Debug, Clone)]
#[require(ForcedVelocity)]
pub struct SpeedDecay(pub f32);
impl Default for SpeedDecay {
    fn default() -> Self {
        Self(0.5)
    }
}

#[derive(Component, Debug, Clone, Default, Deref, DerefMut)]
#[require(LinearVelocity, PrevForcedVelocity)]
pub struct ForcedVelocity(pub Vec2);

#[derive(Component, Debug, Clone, Default)]
struct PrevForcedVelocity(Vec2);

fn update_forced_velocity(
    mut q_forced: Query<(
        Ref<ForcedVelocity>,
        &mut LinearVelocity,
        &mut PrevForcedVelocity,
    )>,
) {
    q_forced
        .par_iter_mut()
        .for_each(|(forced, mut linear_velocity, mut prev)| {
            if forced.is_changed() {
                linear_velocity.0 += forced.0 - prev.0;
                prev.0 = forced.0;
            }
        });
}
fn speed_decay(mut q_velocity: Query<(&SpeedDecay, &mut LinearVelocity, &ForcedVelocity)>) {
    q_velocity
        .par_iter_mut()
        .for_each(|(decay, mut linear_velocity, forced)| {
            let amount = (linear_velocity.0 - forced.0) * decay.0;
            linear_velocity.0 -= amount;
        });
}

#[derive(Component, Debug, Clone, Default)]
#[require(ForcedVelocity)]
pub struct Displacement {
    pub displace: Vec2,
    pub duration: Duration,
    pub notify: Option<Entity>,
}
#[derive(Component, Debug, Clone, Default)]
struct DisplacementInfo {
    start_time: Duration,
    prev_velocity: Vec2,
}
/// This will be sent to `notify` Entity, if any.
#[derive(EntityEvent)]
pub struct DisplacementComplete {
    pub entity: Entity,
}

fn work_displacement(
    commands: ParallelCommands,
    mut q_displacement: Query<(
        Entity,
        &mut ForcedVelocity,
        &Displacement,
        &mut DisplacementInfo,
    )>,
    time: Res<Time>,
) {
    fn derivative(value: f32) -> f32 {
        (QuadraticInOutCurve.sample(value + 1e-6).unwrap()
            - QuadraticInOutCurve.sample(value).unwrap())
            / 1e-6
    }
    q_displacement.par_iter_mut().for_each(
        |(entity, mut forced_velocity, displacement, mut info)| {
            let ratio = (time.elapsed() - info.start_time).as_secs_f32()
                / displacement.duration.as_secs_f32();
            if ratio >= 1.0 {
                **forced_velocity -= info.prev_velocity;
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
            **forced_velocity += new_velocity - info.prev_velocity;
            info.prev_velocity = new_velocity;
        },
    );
}
