use super::*;

pub(super) struct FiledPlugin;

impl Plugin for FiledPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, init_filed);
        app.add_systems(FixedPostUpdate, filed_play_effect);
    }
}

// NOTE: The visibility here is for the WeaponSoundCue child, which emits a warning if without.
#[derive(Component, Debug)]
#[require(FiledTimer, Transform, Visibility)]
pub struct Filed {
    pub interval: f32,
    pub damage: math::ApproxFloat,
    pub order: u8,
    pub size: Vec2,
    pub speed: f32,
}
#[derive(Component, Default)]
struct FiledTimer {
    interval: Timer,
}

/// Marks the filed bullet, for playing splash effect.
#[derive(Component, Default)]
#[require(canum_play::projectile::Projectile)]
pub struct FiledBullet;

impl Default for Filed {
    fn default() -> Self {
        Self {
            interval: 0.1,
            damage: math::ApproxFloat::from(10),
            order: crate::consts::order::PLAYER_PROJ,
            size: Vec2::new(5.0, 10.0),
            speed: 450.0,
        }
    }
}

fn init_filed(
    mut commands: Commands,
    mut q_filed: Query<(Entity, &mut FiledTimer, &Filed), Added<Filed>>,
) {
    for (entity, mut filed_timer, filed) in q_filed.iter_mut() {
        filed_timer.interval = Timer::from_seconds(filed.interval, TimerMode::Repeating);
        commands
            .entity(entity)
            .observe(filed_shoot)
            .insert(children![(
                WeaponSoundCue,
                canum_res::sound::Sound::new("Filed").paused(),
            )]);
    }
}
fn filed_shoot(
    event: On<Attack>,
    mut commands: Commands,
    mut q_filed: Query<(&Filed, &mut FiledTimer, &GlobalTransform, &ChildOf)>,
    q_player: Query<&PlayerShoot>,
    time: Res<Time>,
) {
    let Ok((filed, mut filed_timer, global_transform, parent)) = q_filed.get_mut(event.entity)
    else {
        return;
    };
    let Ok(player_shoot) = q_player.get(parent.0) else {
        return;
    };
    filed_timer.interval.tick(time.delta());
    if filed_timer.interval.just_finished() {
        let direction = Vec2::from_angle(player_shoot.0);
        let position = global_transform.translation().xy() + direction * filed.size.y;
        let transform = Transform::from_translation(Vec3::new(position.x, position.y, 1.0))
            .with_rotation(Quat::from_rotation_z(
                player_shoot.0 - std::f32::consts::FRAC_PI_2,
            ));
        commands.spawn((
            PlayerProjectile,
            FiledBullet,
            crate::health::ContactDamage {
                value: filed.damage.sample(),
                projectile: true,
                order: filed.order,
            },
            transform,
            Animation::new("Filed", filed.size),
            Collider::rectangle(filed.size.x, filed.size.y),
            Mass(0.25),
            LinearVelocity(direction * filed.speed),
        ));
    }
}

fn filed_play_effect(
    commands: ParallelCommands,
    q_filed: Query<(&GlobalTransform, &canum_play::health::ProjectileContacted), With<FiledBullet>>,
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
                    Animation::new("Filed_Splash", vec2(16.0, 16.0)).once_then_despawn(),
                    transform,
                ));
            });
        });
}
