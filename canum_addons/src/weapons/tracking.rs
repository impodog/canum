use super::*;

pub(super) struct TrackingPlugin;

impl Plugin for TrackingPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<Tracking>()
            .on_add(|mut world, HookContext { entity, .. }| {
                let Some(tracking) = world.get::<Tracking>(entity) else {
                    return;
                };
                let tracking = tracking.clone();
                world
                    .commands()
                    .entity(entity)
                    .insert(TrackingArgs {
                        interval: Timer::from_seconds(tracking.interval, TimerMode::Repeating),
                    })
                    .observe(tracking_shoot);
                world.commands().spawn((
                    ChildOf(entity),
                    WeaponSoundCue,
                    Sound::new("Tracking").paused(),
                ));
            });
        app.add_systems(FixedUpdate, (tracking_work, tracking_play_effect));
    }
}

/// Shoots homing missiles with a very low DPS.
#[derive(Component, Debug, Clone)]
#[require(TrackingArgs, Transform, Visibility)]
pub struct Tracking {
    pub interval: f32,
    pub damage: math::ApproxFloat,
    pub order: u8,
    pub size: Vec2,
    pub speed: f32,
}
#[derive(Component, Default)]
struct TrackingArgs {
    interval: Timer,
}

#[derive(Component, Default)]
#[require(canum_play::projectile::Projectile)]
pub struct TrackingBullet {
    pub target: Option<Entity>,
}

impl Default for Tracking {
    fn default() -> Self {
        Self {
            interval: 0.1,
            damage: math::ApproxFloat::from(2.5),
            order: consts::order::PLAYER_PROJ_WEAK,
            size: vec2(7.0, 7.0),
            speed: 360.0,
        }
    }
}

fn tracking_shoot(
    event: On<Attack>,
    mut commands: Commands,
    mut q_tracking: Query<(&Tracking, &mut TrackingArgs, &GlobalTransform, &ChildOf)>,
    q_player_shoot: Query<&PlayerShoot>,
    time: Res<Time>,
) {
    let Ok((tracking, mut args, global_transform, parent)) = q_tracking.get_mut(event.entity)
    else {
        return;
    };
    let Ok(player_shoot) = q_player_shoot.get(parent.0) else {
        return;
    };
    if args.interval.tick(time.delta()).just_finished() {
        let position = global_transform.translation();
        let transform = Transform::from_translation(vec3(position.x, position.y, 1.0))
            .with_rotation(Quat::from_rotation_z(
                player_shoot.0 - std::f32::consts::FRAC_PI_2,
            ));
        commands.spawn((
            PlayerProjectile,
            TrackingBullet::default(),
            crate::health::ContactDamage {
                value: tracking.damage.sample(),
                projectile: true,
                order: tracking.order,
            },
            transform,
            Animation::new("Tracking", tracking.size),
            Collider::circle(tracking.size.x * 0.5),
            Mass(0.2),
            LinearVelocity(Vec2::from_angle(player_shoot.0) * tracking.speed),
        ));
    }
}

fn tracking_work(
    mut q_bullet: Query<(
        &mut TrackingBullet,
        &mut LinearVelocity,
        &mut Rotation,
        &GlobalTransform,
        &canum_play::health::Friendly,
    )>,
    q_transform: Query<&GlobalTransform>,
    q_other: Query<
        (Entity, &GlobalTransform, &canum_play::health::Friendly),
        With<enemy::health::EnemyHealth>,
    >,
) {
    q_bullet.par_iter_mut().for_each(
        |(mut bullet, mut linear_velocity, mut rotation, global_transform, friendly)| {
            let position = global_transform.translation().xy();
            if let Some(target) = bullet.target {
                if let Ok(target_transform) = q_transform.get(target) {
                    let target_position = target_transform.translation().xy();
                    let direction = (target_position - position).normalize_or(vec2(1.0, 0.0));
                    **linear_velocity = direction * linear_velocity.length();
                    *rotation = Rotation::from_sin_cos(-direction.x, direction.y);
                } else {
                    bullet.target = None;
                }
            }
            if bullet.target.is_none() {
                let mut best = Option::<(Entity, f32)>::None;
                for (entity, other_transform, other_friendly) in q_other.iter() {
                    if friendly.0 ^ other_friendly.0 {
                        let other_position = other_transform.translation().xy();
                        let other_distance = other_position.distance_squared(position);
                        if best.is_none_or(|(_, best_distance)| other_distance < best_distance) {
                            best = Some((entity, other_distance));
                        }
                    }
                }
                bullet.target = best.map(|(target, _)| target);
            }
        },
    );
}

fn tracking_play_effect(
    commands: ParallelCommands,
    q_filed: Query<
        (&GlobalTransform, &canum_play::health::ProjectileContacted),
        With<TrackingBullet>,
    >,
    q_transform: Query<&GlobalTransform>,
) {
    q_filed
        .par_iter()
        .for_each(|(global_transform, contacted)| {
            let Some(contacted) = contacted.first() else {
                return;
            };
            let Ok(target_transform) = q_transform.get(*contacted) else {
                return;
            };
            let transform = global_transform.reparented_to(target_transform);
            commands.command_scope(|mut commands| {
                commands.spawn((
                    ChildOf(*contacted),
                    Animation::new("Tracking_Splash", vec2(16.0, 16.0)).once_then_despawn(),
                    transform,
                ));
            });
        });
}
