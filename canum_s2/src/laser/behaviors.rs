use super::*;
use enemy::behavior::*;

pub(super) struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<ShootAndRotate>()
            .on_insert(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(shoot_and_rotate_start)
                    .observe(shoot_and_rotate_begin);
            });
        app.add_systems(
            FixedUpdate,
            shoot_and_rotate_track_player.run_if(in_state(LASER_STATE.clone())),
        );

        app.world_mut()
            .register_component_hooks::<ScreenAttackBase>()
            .on_insert(|mut world, HookContext { entity, .. }| {
                let Some(mut animation) = world.get_mut::<Animation>(entity) else {
                    return;
                };
                animation.color = Color::WHITE.with_alpha(0.3);
            });
        app.world_mut()
            .register_component_hooks::<ScreenAttack>()
            .on_insert(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(screen_attack_start)
                    .observe(screen_attack_choose_behavior);
            });
        app.add_systems(OnEnter(LASER_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(horiz_laser)));
            commands.spawn((SessionOnly, Observer::new(verti_laser)));
        });
        app.add_systems(
            FixedPostUpdate,
            screen_attack_work.run_if(in_state(LASER_STATE.clone())),
        );
        app.add_systems(
            FixedUpdate,
            screen_attack_play_rotation.run_if(in_state(LASER_STATE.clone())),
        );
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct LaserBehaviors;

#[derive(Component, Default)]
#[require(Behavior::new("Laser_ShootAndRotate", 100.8, ["Animation", "Rotation", "ShootAndRotate"]))]
pub struct ShootAndRotate {
    target: Option<Entity>,
    timer: Timer,
}
const LASER_SIZE: Vec2 = vec2(10.0, 32.0);

fn shoot_and_rotate_start(
    event: On<BehaveStart>,
    mut q_timer: Query<&mut ShootAndRotate>,
    mut q_animation: Query<&mut Animation>,
) {
    let Ok(mut timer) = q_timer.get_mut(event.entity) else {
        return;
    };
    timer.target = Some(event.target);
    let Ok(mut animation) = q_animation.get_mut(event.target) else {
        return;
    };
    animation.replace(
        "Laser_Prepare",
        true,
        Some(AnimationInform {
            entity: event.entity,
            index: vec![usize::MAX],
        }),
    );
}

fn shoot_and_rotate_begin(
    event: On<AnimationComplete>,
    mut q_timer: Query<&mut ShootAndRotate>,
    mut commands: Commands,
) {
    let Ok(mut timer) = q_timer.get_mut(event.entity) else {
        return;
    };
    timer.timer = Timer::from_seconds(7.0, TimerMode::Once);
    commands
        .spawn((
            ChildOf(event.entity),
            Transform::from_translation(vec3(30.0, 0.0, 0.0)),
        ))
        .observe(laser_shoot_spawn)
        .insert(canum_tool::weapon::old_laser::LaserLike {
            middle: Animation::new("Laser_AttackMiddle", LASER_SIZE),
            terminal: Animation::new("Laser_AttackTerminal", LASER_SIZE),
            collider: Collider::rectangle(LASER_SIZE.x, LASER_SIZE.y - 4.0),
            length: LASER_SIZE.x,
        });
    commands.spawn((ChildOf(event.entity), Sound::new("Laser_Shoot")));
}

fn laser_shoot_spawn(event: On<canum_tool::weapon::old_laser::LaserSpawn>, mut commands: Commands) {
    commands.entity(event.spawn).insert((
        projectile::Projectile::default().no_dispose(),
        health::ContactDamage {
            value: 50,
            projectile: false,
            order: consts::order::PLAYER_PROJ,
        },
        health::Friendly::UNFRIENDLY,
    ));
}

fn shoot_and_rotate_track_player(
    mut q_timer: Query<(Entity, &mut ShootAndRotate, &GlobalTransform)>,
    mut q_laser: Query<(&mut Rotation, &mut Animation), With<LaserBoss>>,
    q_global_transform: Query<&GlobalTransform>,
    player: Option<Res<player::PrimaryPlayer>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let Some(player) = player else {
        return;
    };
    let Ok(player_transform) = q_global_transform.get(player.0) else {
        return;
    };
    let player_position = player_transform.translation().xy();
    for (entity, mut timer, global_transform) in q_timer.iter_mut() {
        if timer.timer.is_finished() {
            continue;
        }
        let Some(target) = timer.target else {
            continue;
        };
        let Ok((mut rotation, mut animation)) = q_laser.get_mut(target) else {
            continue;
        };
        if timer.timer.tick(time.delta()).just_finished() {
            commands.trigger(BehaveEnd {
                entity,
                cooldown: Duration::from_secs_f32(rand_normal(1.0, 0.2)),
                occupies: occupies![("ShootAndRotate", 6.0)],
            });
            animation.replace("Laser_Static", false, None);
            commands.entity(entity).despawn_children();
            continue;
        }
        let position = global_transform.translation().xy();
        let diff = player_position - position;
        let min_angular_velocity = (108.0 * diff.length_recip() * 2.0).clamp(2.0, 10.0);
        let angle = diff.to_angle();
        let angle_diff =
            normalize_angle_signed(angle - global_transform.rotation().to_euler(EulerRot::XYZ).2);
        let velocity = angle_diff * 2.2;
        let velocity = velocity.signum()
            * velocity
                .abs()
                .clamp(min_angular_velocity, min_angular_velocity + 1.2);
        // FIXME Why no rotation???
        *rotation = rotation.add_angle_fast(velocity * time.delta_secs());
    }
}

#[derive(Component)]
#[require(Behavior::new("Laser_ScreenAttack", 1.2, ["ScreenAttack", "Animation", "Rotation"]))]
pub struct ScreenAttack {
    start_timer: Timer,
    rotate_timer: Timer,
    target: Option<Entity>,
}

impl Default for ScreenAttack {
    fn default() -> Self {
        let mut value = Self {
            start_timer: Timer::from_seconds(0.2, TimerMode::Once),
            rotate_timer: Timer::from_seconds(0.8, TimerMode::Once),
            target: None,
        };
        value.rotate_timer.finish();
        value
    }
}

#[derive(Component)]
#[require(
    Animation::new("Laser_Screen", vec2(20.0, 16.0)),
    RigidBody::Static,
    Sensor,
    Collider::rectangle(20.0, 15.0),
    ColliderDisabled,
    health::Friendly(false),
    health::ContactDamage { value: 100, projectile: false, order: consts::order::ENEMY_PROJ },
)]
struct ScreenAttackBase {
    transparent: Timer,
    disappear: Timer,
}
impl Default for ScreenAttackBase {
    fn default() -> Self {
        Self {
            transparent: Timer::from_seconds(1.2, TimerMode::Once),
            disappear: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }
}

fn screen_attack_work(
    commands: ParallelCommands,
    mut q_screen_attack: Query<(Entity, &mut ScreenAttackBase, &mut Animation)>,
    time: Res<Time>,
) {
    q_screen_attack
        .par_iter_mut()
        .for_each(|(entity, mut timer, mut animation)| {
            if timer.transparent.is_finished() {
                if timer.disappear.tick(time.delta()).just_finished() {
                    commands.command_scope(|mut commands| {
                        commands.entity(entity).despawn();
                    });
                }
            } else if timer.transparent.tick(time.delta()).just_finished() {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).remove::<ColliderDisabled>();
                });
                animation.color = Color::WHITE;
            }
        });
}

#[derive(Event)]
struct HorizLaser(f32);

#[derive(Event)]
struct VertiLaser(f32);

const SCREEN_ATTACK_SIZE: Vec2 = vec2(20.0, 16.0);

fn horiz_laser(event: On<HorizLaser>, mut commands: Commands) {
    let mut x = -CONFIG.display.half_virtual_size.0 + SCREEN_ATTACK_SIZE.x * 0.5;
    while x < CONFIG.display.half_virtual_size.0 {
        commands.spawn((
            ScreenAttackBase::default(),
            Transform::from_translation(vec3(x, event.0, 1.0)),
        ));
        x += SCREEN_ATTACK_SIZE.x;
    }
}

fn verti_laser(event: On<VertiLaser>, mut commands: Commands) {
    let mut y = -CONFIG.display.half_virtual_size.1 + SCREEN_ATTACK_SIZE.y * 0.5;
    while y < CONFIG.display.half_virtual_size.1 {
        commands.spawn((
            ScreenAttackBase::default(),
            Transform::from_translation(vec3(event.0, y, 1.1))
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        ));
        y += SCREEN_ATTACK_SIZE.x;
    }
}

fn screen_attack_start(event: On<BehaveStart>, mut q_screen_attack: Query<&mut ScreenAttack>) {
    let Ok(mut screen_attack) = q_screen_attack.get_mut(event.entity) else {
        return;
    };
    screen_attack.start_timer.reset();
    screen_attack.rotate_timer.reset();
    screen_attack.target = Some(event.target);
}

fn screen_attack_play_rotation(
    mut q_screen_attack: Query<(Entity, &mut ScreenAttack)>,
    mut q_rotation: Query<&mut Rotation>,
    time: Res<Time>,
    mut commands: Commands,
) {
    const START_ROTATION: f32 = 1.0;
    const MAIN_ROTATION: f32 = -std::f32::consts::TAU;

    fn derivative(x: f32) -> f32 {
        (CubicInOutCurve.sample(x + 1e-6).unwrap_or(1.0) - CubicInOutCurve.sample(x).unwrap_or(1.0))
            / 1e-6
    }

    let delta_secs = time.delta_secs();

    for (entity, mut timers) in q_screen_attack.iter_mut() {
        let Some(target) = timers.target else {
            continue;
        };
        let Ok(mut rotation) = q_rotation.get_mut(target) else {
            return;
        };
        if !timers.start_timer.is_finished() {
            *rotation = rotation.add_angle_fast(
                derivative(timers.start_timer.fraction()) * START_ROTATION * delta_secs,
            );
            timers.start_timer.tick(time.delta());
        } else {
            *rotation = rotation.add_angle_fast(
                derivative(timers.rotate_timer.fraction()) * MAIN_ROTATION * delta_secs,
            );
            if timers.rotate_timer.tick(time.delta()).just_finished() {
                commands.spawn(Sound::new("Laser_ScreenAttack"));
                commands.trigger(BehaveEnd {
                    entity,
                    cooldown: Duration::from_secs_f32(1.5),
                    occupies: occupies![("ScreenAttack", rand_normal(1.0, 1.0).clamp(0.5, 2.0))],
                });
            }
        }
    }
}

fn screen_attack_choose_behavior(
    _event: On<BehaveStart>,
    mut commands: Commands,
    q_player: Query<&GlobalTransform>,
    primary_player: Option<Res<player::PrimaryPlayer>>,
) {
    const LAYOUT_COUNT: u32 = 7;
    let Some(primary_player) = primary_player else {
        return;
    };
    let layout_index = rand::random_range(0..LAYOUT_COUNT);
    match layout_index {
        0 => {
            let mut position =
                CONFIG.display.half_virtual_size.0 + rand_normal(0.0, 10.0).max(-10.0);
            let spacing = rand_normal(70.0, 5.0);
            while position > -CONFIG.display.half_virtual_size.0 {
                commands.trigger(VertiLaser(position));
                position -= spacing;
            }
        }
        1 => {
            let mut position =
                CONFIG.display.half_virtual_size.1 + rand_normal(0.0, 10.0).max(-10.0);
            let spacing = rand_normal(70.0, 4.0);
            while position > -CONFIG.display.half_virtual_size.1 {
                commands.trigger(HorizLaser(position));
                position -= spacing;
            }
        }
        2 => {
            let sgn = rand_sign();
            let mut position = -175.0;
            while position < CONFIG.display.half_virtual_size.0 {
                commands.trigger(VertiLaser(position * sgn));
                position += SCREEN_ATTACK_SIZE.y;
            }
        }
        3 => {
            let sgn = rand_sign();
            let mut position = -90.0;
            while position < CONFIG.display.half_virtual_size.1 {
                commands.trigger(HorizLaser(position * sgn));
                position += SCREEN_ATTACK_SIZE.y;
            }
        }
        4 | 5 => {
            let begin = rand::random_range(0.0..std::f32::consts::PI);
            let mut current = begin;
            let mut flag = false;
            let spacing = std::f32::consts::FRAC_PI_8;
            let mut z = 1.1;
            // For each line of laser by rotation.
            loop {
                if (current - begin).abs() < 1e-6 {
                    if flag {
                        break;
                    }
                    flag = true;
                }
                // Spawn the array of laser nodes.
                let mut position = vec2(0.0, 0.0);
                let diff = SCREEN_ATTACK_SIZE.x * Vec2::from_angle(current);
                while CONFIG.display.screen_rect.contains(position) {
                    commands.spawn((
                        ScreenAttackBase::default(),
                        Transform::from_translation(vec3(position.x, position.y, z))
                            .with_rotation(Quat::from_rotation_z(current)),
                    ));
                    if position.x != 0.0 {
                        commands.spawn((
                            ScreenAttackBase::default(),
                            Transform::from_translation(vec3(-position.x, -position.y, z))
                                .with_rotation(Quat::from_rotation_z(current)),
                        ));
                    }
                    position += diff;
                }
                current = normalize_angle(current + spacing);
                z += 0.01;
            }
        }
        6 => {
            let Ok(global_transform) = q_player.get(primary_player.0) else {
                return;
            };
            let angle = global_transform.translation().xy().to_angle();
            let mut z = 1.1;
            for i in -6..=6 {
                let current = i as f32 * 0.1 + angle;
                // Spawn the array of laser nodes.
                let mut position = vec2(0.0, 0.0);
                let diff = SCREEN_ATTACK_SIZE.x * Vec2::from_angle(current);
                while CONFIG.display.screen_rect.contains(position) {
                    commands.spawn((
                        ScreenAttackBase::default(),
                        Transform::from_translation(vec3(position.x, position.y, z))
                            .with_rotation(Quat::from_rotation_z(current)),
                    ));
                    if position.x != 0.0 {
                        commands.spawn((
                            ScreenAttackBase::default(),
                            Transform::from_translation(vec3(-position.x, -position.y, z))
                                .with_rotation(Quat::from_rotation_z(current)),
                        ));
                    }
                    position += diff;
                }
                z += 0.01;
            }
        }
        _ => warn!("Undefined layout index {layout_index}"),
    }
}
