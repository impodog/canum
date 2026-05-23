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
                world
                    .commands()
                    .entity(entity)
                    .insert(DisplacementInfo { start_time });
            });
        app.add_systems(FixedUpdate, (work_displacement, rotation_around));
    }
}

/// Calls for the parent to move in speed linked to a curve, with a fixed displacement.
#[derive(Component, Debug, Clone)]
#[require(PartialVelocity::unlinked())]
pub struct Displacement {
    /// The curve must be derivable for (0.0, 1.0), while 0.0 maps to 0.0, 1.0 maps to 1.0.
    pub curve: fn(f32) -> f32,
    pub displace: Vec2,
    pub duration: Duration,
    pub notify: Option<Entity>,
}
impl Default for Displacement {
    fn default() -> Self {
        Self {
            curve: |x| QuadraticOutCurve.sample(x).unwrap(),
            displace: Vec2::ZERO,
            duration: Duration::from_secs(1),
            notify: None,
        }
    }
}

#[derive(Component, Debug, Clone)]
struct DisplacementInfo {
    start_time: Duration,
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
        &Displacement,
        &DisplacementInfo,
        &mut PartialVelocity,
    )>,
    time: Res<Time>,
) {
    fn derivative(curve: fn(f32) -> f32, value: f32) -> f32 {
        let next = (value + 1e-6).min(1.0);
        let ans = (curve(next) - curve(value)) / (next - value);
        if ans.is_finite() { ans } else { 0.0 }
    }
    q_displacement
        .par_iter_mut()
        .for_each(|(entity, displacement, info, mut partial_velocity)| {
            let ratio = (time.elapsed_secs() - info.start_time.as_secs_f32())
                / displacement.duration.as_secs_f32();
            let ratio = ratio.clamp(0.0, 1.0);
            if ratio >= 1.0 {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                    if let Some(notify_entity) = displacement.notify {
                        commands.trigger(DisplacementComplete {
                            entity: notify_entity,
                        });
                    }
                });
                return;
            }
            let new_velocity = derivative(displacement.curve, ratio)
                / displacement.duration.as_secs_f32()
                * displacement.displace;
            **partial_velocity = new_velocity;
        });
}

/// Adds fixed rotation around this entity.
#[derive(Component)]
#[require(Transform, PartialVelocity::unlinked())]
pub struct RotationAround {
    pub around: Entity,
    /// ccw is positive.
    pub angular_velocity: f32,
}

fn rotation_around(
    mut q_rotation: Query<(&mut PartialVelocity, &RotationAround, &GlobalTransform)>,
    q_transform: Query<&GlobalTransform>,
    time: Res<Time>,
) {
    q_rotation
        .par_iter_mut()
        .for_each(|(mut partial_velocity, rotation, transform)| {
            let Ok(around_transform) = q_transform.get(rotation.around) else {
                return;
            };
            let position = transform.translation().xy();
            let around = around_transform.translation().xy();
            let normal = position - around;
            let next_position = around
                + Vec2::from_angle(rotation.angular_velocity * time.delta_secs()).rotate(normal);
            partial_velocity.velocity = (next_position - position) / time.delta_secs();
        });
}
