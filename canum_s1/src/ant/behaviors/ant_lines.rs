use super::*;

pub(super) struct AntLinesPlugin;

impl Plugin for AntLinesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            spawn_ant_runner_line.run_if(in_state(super::super::ANT_STATE.clone())),
        );
        app.add_systems(
            FixedPreUpdate,
            update_ant_runner_line.run_if(in_state(super::super::ANT_STATE.clone())),
        );
    }
}

#[derive(Component)]
#[require(
    enemy::attack::Minion,
    projectile::RemoveOutOfBounds {distance_scale: 0.5},
    Animation::new("Ant_SubAntPlane", Vec2::new(40.0, 40.0)),
    movements::ForcedVelocity,
    movements::SpeedDecay(0.05),
    Collider::rectangle(20.0, 30.0),
    Mass(5.0)
)]
pub(super) struct SubAntRunner {
    pub(super) direction: f32,
}

#[derive(Component, Debug)]
struct Line {
    related: Entity,
    start: Vec2,
    displace: Vec2,
    // These two are percentage.
    current_start: f32,
    current_end: f32,
}

fn spawn_ant_runner_line(
    mut commands: Commands,
    mut q_runner: Query<
        (
            Entity,
            &SubAntRunner,
            &mut Transform,
            &mut movements::ForcedVelocity,
        ),
        Added<SubAntRunner>,
    >,
) {
    for (related, runner, mut transform, mut forced_velocity) in q_runner.iter_mut() {
        let translation = transform.translation;
        transform.rotation = Quat::from_rotation_z(runner.direction - std::f32::consts::FRAC_PI_2);
        **forced_velocity = Vec2::from_angle(runner.direction) * 250.0;
        let start = translation.xy();
        let displace = Vec2::from_angle(runner.direction) * CONFIG.display.virtual_diagonal * 1.5;
        commands.spawn((
            SessionOnly,
            Line {
                related,
                start,
                displace,
                current_start: 0.0,
                current_end: 0.0,
            },
            Transform::from_translation(vec3(0.0, 0.0, 14.17)),
        ));
    }
}

fn update_ant_runner_line(
    mut commands: Commands,
    mut q_line: Query<(Entity, &mut Line)>,
    mut gizmos: Gizmos,
    q_transform: Query<&GlobalTransform>,
) {
    const LINE_COLOR: Color = Color::srgba(0.3, 0.3, 0.2, 0.2);
    for (entity, mut line) in q_line.iter_mut() {
        let Ok(target_transform) = q_transform.get(line.related) else {
            commands.entity(entity).despawn();
            continue;
        };
        let target_displace = target_transform.translation().xy() - line.start;
        line.current_start = target_displace.dot(line.displace) / line.displace.length_squared();
        line.current_start = line.current_start.clamp(0.0, 1.0);
        if line.current_end < 1.0 {
            line.current_end += 0.015;
            line.current_end = line.current_end.max(line.current_start);
        }
        let diff = target_displace - target_displace.project_onto(line.displace);
        line.start += diff;
        gizmos.line_2d(
            line.start
                + line.current_start * line.displace
                + 30.0 * line.displace.normalize_or_zero(),
            line.start + line.current_end * line.displace,
            LINE_COLOR,
        );
    }
}
