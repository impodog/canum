use super::*;

pub(super) struct SpreadPlugin;

impl Plugin for SpreadPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().register_component_hooks::<Spread>().on_add(
            |mut world, HookContext { entity, .. }| {
                let Some(spread) = world.get::<Spread>(entity) else {
                    return;
                };
                let spread = spread.clone();
                let mut commands = world.commands();
                commands
                    .entity(entity)
                    .insert(SpreadArgs {
                        interval: Timer::from_seconds(spread.interval, TimerMode::Repeating),
                        spread_angle: spread.base_spread_angle,
                    })
                    .observe(spread_shoot);
            },
        );
        app.add_systems(FixedUpdate, spread_update);
        app.add_systems(FixedPostUpdate, spread_play_effect);
    }
}

/// Shoots three spreading bullets with high DPS (if all hit), but the spread angle increases as player gains speed.
#[derive(Component, Debug, Clone)]
#[require(SpreadArgs, Transform, Visibility)]
pub struct Spread {
    pub interval: f32,
    pub damage: math::ApproxFloat,
    pub order: u8,
    pub size: Vec2,
    pub speed: f32,
    pub max_spread_angle: f32,
    pub base_spread_angle: f32,
    pub increment_ratio: f32,
}
#[derive(Component, Default)]
struct SpreadArgs {
    interval: Timer,
    spread_angle: f32,
}

/// Marks the bullet as Spread.
#[derive(Component, Default)]
#[require(canum_play::projectile::Projectile)]
pub struct SpreadBullet;

impl Default for Spread {
    fn default() -> Self {
        Self {
            interval: 0.25,
            damage: math::ApproxFloat::from(10),
            order: consts::order::PLAYER_PROJ,
            size: vec2(6.0, 8.0),
            speed: 375.0,
            max_spread_angle: (45.0_f32).to_radians(),
            base_spread_angle: (5.0_f32).to_radians(),
            increment_ratio: 80.0_f32.recip() * (1.0_f32).to_radians(),
        }
    }
}

fn spread_shoot(
    event: On<Attack>,
    mut commands: Commands,
    mut q_spread: Query<(&Spread, &mut SpreadArgs, &GlobalTransform, &ChildOf)>,
    q_player: Query<&PlayerShoot>,
    time: Res<Time>,
) {
    let Ok((spread, mut spread_args, global_transform, parent)) = q_spread.get_mut(event.entity)
    else {
        return;
    };
    let Ok(player_shoot) = q_player.get(parent.0) else {
        return;
    };
    if spread_args.interval.tick(time.delta()).just_finished() {
        for angle_displacement in [-spread_args.spread_angle, 0.0, spread_args.spread_angle] {
            let angle = player_shoot.0 + angle_displacement;
            let direction = Vec2::from_angle(angle);
            let position = global_transform.translation().xy();
            let transform = Transform::from_translation(vec3(position.x, position.y, 1.0))
                .with_rotation(Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2));
            commands.spawn((
                PlayerProjectile,
                SpreadBullet,
                crate::health::ContactDamage {
                    value: spread.damage.sample(),
                    projectile: true,
                    order: spread.order,
                },
                transform,
                Animation::new("Spread", spread.size),
                Collider::rectangle(spread.size.x, spread.size.y + 1.0),
                Mass(0.37),
                LinearVelocity(direction * spread.speed),
            ));
        }
        commands.spawn(Sound::new("Spread"));
    }
}

fn spread_update(
    mut q_spread: Query<(&Spread, &mut SpreadArgs, &ChildOf)>,
    q_player: Query<&LinearVelocity>,
    time: Res<Time>,
) {
    for (spread, mut spread_arg, parent) in q_spread.iter_mut() {
        let Ok(velocity) = q_player.get(parent.0) else {
            return;
        };
        let target_angle = (velocity.length() * spread.increment_ratio + spread.base_spread_angle)
            .min(spread.max_spread_angle);
        let diff = target_angle - spread_arg.spread_angle;
        // It would take 0.2s to fully change to target angle.
        spread_arg.spread_angle += diff * time.delta_secs() * 5.0;
        if diff >= 0.0 {
            spread_arg.spread_angle = spread_arg.spread_angle.min(target_angle);
        } else {
            spread_arg.spread_angle = spread_arg.spread_angle.max(target_angle);
        }
    }
}

fn spread_play_effect(
    commands: ParallelCommands,
    q_filed: Query<
        (&GlobalTransform, &canum_play::health::ProjectileContacted),
        With<SpreadBullet>,
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
                    Animation::new("Spread_Splash", vec2(16.0, 16.0)).once_then_despawn(),
                    transform,
                ));
            });
        });
}
