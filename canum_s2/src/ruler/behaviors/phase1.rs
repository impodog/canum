use super::*;

pub(super) struct Phase1Plugin;

impl Plugin for Phase1Plugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<RulerPhase1>()
            .on_add(ruler_phase1_hook);
        app.world_mut()
            .register_component_hooks::<Swipe>()
            .on_add(swipe_hook);
        app.world_mut()
            .register_component_hooks::<LaserAttack>()
            .on_add(laser_attack_hook);
        app.world_mut()
            .register_component_hooks::<BounceBall>()
            .on_add(bounce_ball_hook);
        app.world_mut()
            .register_component_hooks::<Ball>()
            .on_add(ball_hook);
        app.add_systems(
            FixedUpdate,
            (
                swipe_align_with_player,
                laser_attack_platform,
                bounce_ball_shoot_ball,
            )
                .in_set(RulerSet),
        );
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct RulerPhase1;

fn ruler_phase1_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let mut commands = world.commands();
    commands.spawn((ChildOf(entity), Swipe::default()));
    commands.spawn((ChildOf(entity), LaserAttack::default()));
    commands.spawn((ChildOf(entity), BounceBall::default()));
}

#[derive(Component, Default)]
#[require(Behavior::new("Ruler_Swipe", 0.1, ["Main", "Swipe"]))]
pub struct Swipe {
    pub target: Option<Entity>,
    pub status: SwipeStatus,
    pub is_called: bool,
}
#[derive(Debug, Clone, Default)]
pub enum SwipeStatus {
    #[default]
    Inactive,
    GoToBottom,
    /// The time remaining for aligning and the velocity component.
    Align(Timer, Entity),
    Warning,
    Swiping,
}

fn swipe_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world
        .commands()
        .entity(entity)
        .observe(swipe_start)
        .observe(swipe_displacement_complete)
        .observe(swipe_warning_complete);
    if let Some(mut transform) = world.get_mut::<Transform>(entity) {
        // Anchored to the bottom of the ruler.
        transform.translation.y -= INITIAL_HEIGHT * 0.5;
    }
}

fn swipe_start(
    event: On<BehaveStart>,
    mut commands: Commands,
    q_transform: Query<&GlobalTransform>,
    mut q_swipe: Query<&mut Swipe>,
) {
    let Ok(mut swipe) = q_swipe.get_mut(event.entity) else {
        return;
    };
    swipe.target = Some(event.target);
    swipe.status = SwipeStatus::GoToBottom;
    swipe.is_called = event.caller.is_some();

    let Ok(transform) = q_transform.get(event.entity) else {
        return;
    };
    let position = transform.translation().xy();
    let displace = vec2(0.0, -CONFIG.display.half_virtual_size.1 - position.y);
    commands.spawn((
        ChildOf(event.target),
        enemy::movements::Displacement {
            curve: |x| QuadraticInOutCurve.sample(x).unwrap(),
            displace,
            duration: Duration::from_secs_f32(displace.length() / 180.0),
            notify: Some(event.entity),
        },
        canum_fx::physics::ParentColliderDisabled,
    ));
}
fn swipe_displacement_complete(
    event: On<enemy::movements::DisplacementComplete>,
    mut q_swipe: Query<&mut Swipe>,
    mut commands: Commands,
) {
    let Ok(mut swipe) = q_swipe.get_mut(event.entity) else {
        return;
    };
    match swipe.status {
        SwipeStatus::GoToBottom => {
            let Some(target) = swipe.target else {
                return;
            };
            let velocity = commands
                .spawn((
                    ChildOf(target),
                    movements::PartialVelocity::linked(event.entity),
                ))
                .id();
            let align_time = if swipe.is_called {
                rand_normal(0.5, 0.05)
            } else {
                rand_normal(2.0, 0.3).clamp(1.5, 2.3)
            };
            swipe.status =
                SwipeStatus::Align(Timer::from_seconds(align_time, TimerMode::Once), velocity);
        }
        SwipeStatus::Swiping => {
            swipe.status = SwipeStatus::Inactive;
            commands.trigger(BehaveEnd {
                entity: event.entity,
                cooldown: Duration::from_secs_f32(rand_normal(1.0, 0.12)),
                occupies: occupies![("Swipe", rand_normal(8.0, 0.4))],
            });
        }
        _ => {
            warn!("Unknown status in behavior Ruler_Swipe: {:?}", swipe.status);
        }
    }
}

fn swipe_align_with_player(
    mut q_swipe: Query<(Entity, &mut Swipe)>,
    time: Res<Time>,
    player: Option<Res<player::PrimaryPlayer>>,
    q_transform: Query<&Transform>,
    q_global_transform: Query<&GlobalTransform>,
    mut q_velocity: Query<&mut movements::PartialVelocity>,
    mut commands: Commands,
) {
    let Some(player) = player else {
        return;
    };
    for (entity, mut swipe) in q_swipe.iter_mut() {
        let Some(target) = swipe.target else {
            return;
        };
        if let SwipeStatus::Align(ref mut timer, velocity_entity) = swipe.status {
            let Ok(player_transform) = q_global_transform.get(player.0) else {
                return;
            };
            let player_position = player_transform.translation().xy();

            if timer.tick(time.delta()).just_finished() {
                swipe.status = SwipeStatus::Warning;
                commands.entity(velocity_entity).despawn();
                let Ok(transform) = q_transform.get(target) else {
                    return;
                };
                let mut new_transform = *transform;
                new_transform.translation.z -= 0.1;
                let effect_entity = commands
                    .spawn((
                        Animation::new("Ruler_Ruler", SIZE),
                        canum_fx::emphasis::ExpandAndFadeOut::default().with_time(0.35),
                        new_transform,
                    ))
                    .id();
                commands
                    .spawn(canum_fx::util::DespawnCheck::new(effect_entity).with_notify(entity));
                commands.spawn(Sound::new("Windy_Warning"));
            } else {
                let Ok(transform) = q_global_transform.get(entity) else {
                    return;
                };
                let Ok(mut velocity) = q_velocity.get_mut(velocity_entity) else {
                    return;
                };
                let position = transform.translation().xy();
                let diff = player_position.x - position.x;
                if diff.abs() < 10.0 {
                    velocity.x = 0.0;
                } else if swipe.is_called {
                    velocity.x = diff.signum() * (diff.abs() * 5.0).clamp(160.0, 400.0);
                } else {
                    velocity.x = diff.signum() * (diff.abs() * 4.0).clamp(160.0, 400.0);
                }
            }
        }
    }
}

fn swipe_warning_complete(
    event: On<canum_fx::util::DespawnObserved>,
    mut q_swipe: Query<&mut Swipe>,
    mut commands: Commands,
) {
    let Ok(mut swipe) = q_swipe.get_mut(event.entity) else {
        return;
    };
    let Some(target) = swipe.target else {
        return;
    };
    swipe.status = SwipeStatus::Swiping;
    commands.spawn((
        ChildOf(target),
        enemy::movements::Displacement {
            curve: canum_fx::quadratic_curve!(2.0, 0.5),
            displace: vec2(0.0, CONFIG.display.screen_size.y - INITIAL_HEIGHT * 2.0),
            duration: Duration::from_secs_f32(0.38),
            notify: Some(event.entity),
        },
    ));
    commands.spawn(Sound::new("Bread_Dash"));
}

#[derive(Component, Default)]
#[require(Behavior::new("Ruler_LaserAttack", 1.0, ["Main", "LaserAttack"]))]
pub struct LaserAttack {
    target: Option<Entity>,
    x_direction: f32,
    state: LaserAttackState,
}
#[derive(Default, Debug, Clone, Copy)]
enum LaserAttackState {
    #[default]
    ToCorner,
    Attack,
}

#[derive(Component, Default)]
struct LaserAttackSoundEffect;

#[derive(Component)]
#[require(
    Animation::new("Wcat_Bar", vec2(104.0, 15.6)),
    Collider::rectangle(104.0, 15.6),
    RigidBody::Static,
    health::CollidePlayerOnly
)]
struct LaserAttackPlatform {
    timer: Timer,
    fade_in: bool,
}
impl Default for LaserAttackPlatform {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            fade_in: true,
        }
    }
}

fn laser_attack_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world
        .commands()
        .entity(entity)
        .observe(laser_attack_start)
        .observe(laser_attack_go);
}

fn laser_attack_platform(
    mut q_platform: Query<(Entity, &mut LaserAttackPlatform, &mut Sprite)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut platform, mut sprite) in q_platform.iter_mut() {
        if platform.timer.is_finished() {
            continue;
        };
        platform.timer.tick(time.delta());
        if platform.timer.just_finished() {
            sprite
                .color
                .set_alpha(if platform.fade_in { 1.0 } else { 0.0 });
            if !platform.fade_in {
                commands.entity(entity).despawn();
            }
        } else {
            let fraction = platform.timer.fraction();
            sprite.color.set_alpha(if platform.fade_in {
                fraction
            } else {
                1.0 - fraction
            });
        }
    }
}

fn laser_attack_start(
    event: On<BehaveStart>,
    mut q_laser_attack: Query<&mut LaserAttack>,
    mut commands: Commands,
    q_transform: Query<&GlobalTransform>,
) {
    let Ok(mut laser_attack) = q_laser_attack.get_mut(event.entity) else {
        return;
    };
    laser_attack.target = Some(event.target);
    laser_attack.x_direction = rand_sign();
    laser_attack.state = LaserAttackState::ToCorner;

    let Ok(transform) = q_transform.get(event.entity) else {
        return;
    };
    let position = transform.translation().xy();
    let target_position = vec2(
        (CONFIG.display.half_virtual_size.0 - SIZE.x * 0.5) * -laser_attack.x_direction,
        (CONFIG.display.half_virtual_size.1 - SIZE.y * 0.5) * rand_sign(),
    );
    let displace = target_position - position;
    commands.spawn((
        ChildOf(event.target),
        enemy::movements::Displacement {
            curve: canum_fx::quadratic_curve!(2.0, 0.5),
            displace,
            duration: Duration::from_secs_f32(1.0),
            notify: Some(event.entity),
        },
        canum_fx::physics::ParentColliderDisabled,
    ));
    let x_lim = CONFIG.display.half_virtual_size.0 * 0.75;
    let y_lim = CONFIG.display.half_virtual_size.1 * 0.4;
    commands.spawn((
        LaserAttackPlatform::default(),
        Sprite {
            color: Color::Srgba(Srgba::WHITE.with_alpha(0.0)),
            ..default()
        },
        Transform::from_translation(vec3(
            rand_range(-x_lim..x_lim),
            rand_range(-y_lim..y_lim),
            0.0,
        )),
    ));
}

#[allow(clippy::type_complexity)]
fn laser_attack_go(
    event: On<enemy::movements::DisplacementComplete>,
    mut q_laser_attack: Query<&mut LaserAttack>,
    mut commands: Commands,
    q_despawn: Query<Entity, Or<(With<super::laser::RulerLaser>, With<LaserAttackSoundEffect>)>>,
    mut q_platform: Query<&mut LaserAttackPlatform>,
) {
    let Ok(mut laser_attack) = q_laser_attack.get_mut(event.entity) else {
        return;
    };
    let Some(target) = laser_attack.target else {
        return;
    };
    match laser_attack.state {
        LaserAttackState::ToCorner => {
            let mut batch = Vec::new();
            let mut current = 10.0;
            while current + 5.0 < SIZE.x * 0.5 {
                batch.push((
                    super::laser::RulerLaser {
                        direction: vec2(0.0, 1.0),
                        double: true,
                    },
                    Transform::from_translation(vec3(current, 0.0, -0.1)),
                    canum_fx::transform::Follow::new(target),
                ));
                batch.push((
                    super::laser::RulerLaser {
                        direction: vec2(0.0, 1.0),
                        double: true,
                    },
                    Transform::from_translation(vec3(-current, 0.0, -0.1)),
                    canum_fx::transform::Follow::new(target),
                ));
                current += 20.0;
            }
            commands.spawn_batch(batch);
            let displace = vec2(
                (CONFIG.display.screen_size.x - SIZE.x) * laser_attack.x_direction,
                0.0,
            );
            commands.spawn((LaserAttackSoundEffect, Sound::new("Ruler_Laser")));
            commands.spawn((
                ChildOf(target),
                enemy::movements::Displacement {
                    curve: |x| x,
                    displace,
                    duration: Duration::from_secs_f32(displace.x.abs() / 250.0),
                    notify: Some(event.entity),
                },
            ));
            laser_attack.state = LaserAttackState::Attack;
        }
        LaserAttackState::Attack => {
            for laser in q_despawn.iter() {
                commands.entity(laser).despawn();
            }
            for mut platform in q_platform.iter_mut() {
                platform.timer.reset();
                platform.fade_in = false;
            }
            commands.trigger(BehaveEnd {
                entity: event.entity,
                cooldown: Duration::from_secs_f32(0.5),
                occupies: occupies![("LaserAttack", rand_normal(9.0, 1.0).clamp(7.0, 9.0))],
            });
            if rand_bool(0.5) {
                commands.trigger(BehaveQueue::new(event.entity, "Ruler_Swipe"));
            }
        }
    }
}

#[derive(Component, Default)]
#[require(Behavior::new("Ruler_BounceBall", 1.0, ["BounceBall", "Main"]))]
struct BounceBall {
    target: Option<Entity>,
    remaining_times: i8,
    sign: f32,
    start_bouncing: bool,
    interval: Timer,
    wait: Timer,
    related: Vec<Entity>,
}

const BALL_RADIUS: f32 = 20.0;
#[derive(Component)]
#[require(
    Animation::new("Ruler_Ball", vec2(BALL_RADIUS, BALL_RADIUS)),
    RigidBody::Dynamic,
    LockedAxes::ROTATION_LOCKED,
    Collider::circle(BALL_RADIUS * 0.9),
    Mass(10.0),
    Restitution {coefficient: 1.0, combine_rule: CoefficientCombine::Max},
    health::Friendly(false),
    health::ContactDamage {value: 50, projectile: false, order: consts::order::ENEMY_PROJ},
    CollisionEventsEnabled
)]
struct Ball {
    bounce_times: i8,
}
impl Default for Ball {
    fn default() -> Self {
        Self { bounce_times: 2 }
    }
}

fn bounce_ball_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world
        .commands()
        .entity(entity)
        .observe(bounce_ball_start)
        .observe(bounce_ball_start_bouncing);
}

fn ball_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world.commands().entity(entity).observe(ball_bounce);
}

fn ball_bounce(
    event: On<CollisionEnd>,
    mut commands: Commands,
    mut q_ball: Query<&mut Ball>,
    q_boundary: Query<&GlobalTransform, With<setup::Boundaries>>,
) {
    // If colliding only the down boundaries
    if let Ok(global_transform) = q_boundary.get(event.collider2)
        && global_transform.translation().y.abs() > -1e-3
    {
        let Ok(mut ball) = q_ball.get_mut(event.collider1) else {
            return;
        };
        ball.bounce_times = ball.bounce_times.saturating_sub(1);
        if ball.bounce_times <= 0 {
            commands
                .entity(event.collider1)
                .insert(projectile::NoCollideBoundary);
        }
    }
}

fn bounce_ball_start(
    event: On<BehaveStart>,
    mut q_bounce_ball: Query<(&GlobalTransform, &mut BounceBall)>,
    mut commands: Commands,
) {
    let Ok((global_transform, mut bounce_ball)) = q_bounce_ball.get_mut(event.entity) else {
        return;
    };
    bounce_ball.target = Some(event.target);
    bounce_ball.remaining_times = rand_range(6..=8);
    bounce_ball.start_bouncing = false;
    bounce_ball.interval = Timer::from_seconds(rand_normal(1.0, 0.14), TimerMode::Repeating);
    bounce_ball.wait = Timer::from_seconds(rand_normal(3.0, 0.25), TimerMode::Once);
    if bounce_ball.sign == 0.0 {
        bounce_ball.sign = 1.0;
    } else {
        bounce_ball.sign = -bounce_ball.sign;
    }

    let position = global_transform.translation().xy();
    commands.spawn((
        ChildOf(event.target),
        enemy::movements::Displacement {
            curve: |x| QuadraticInOutCurve.sample(x).unwrap(),
            displace: vec2(
                0.0,
                (CONFIG.display.half_virtual_size.1 - SIZE.y * 0.5) * bounce_ball.sign - position.y,
            ),
            duration: Duration::from_secs_f32(0.8),
            notify: Some(event.entity),
        },
        canum_fx::physics::ParentColliderDisabled,
    ));
}

fn bounce_ball_start_bouncing(
    event: On<enemy::movements::DisplacementComplete>,
    mut q_bounce_ball: Query<(&GlobalTransform, &mut BounceBall)>,
    mut commands: Commands,
) {
    let Ok((global_transform, mut bounce_ball)) = q_bounce_ball.get_mut(event.entity) else {
        return;
    };
    let Some(target) = bounce_ball.target else {
        return;
    };
    if !bounce_ball.start_bouncing {
        bounce_ball.start_bouncing = true;
        for i in 1..=5 {
            let child = commands
                .spawn((
                    super::laser::RulerLaser {
                        direction: vec2(1.0, 0.0),
                        double: false,
                    },
                    Transform::from_translation(vec3(
                        -CONFIG.display.half_virtual_size.0 + 0.1,
                        (CONFIG.display.half_virtual_size.1 - 17.0 * i as f32) * bounce_ball.sign,
                        0.0,
                    )),
                ))
                .id();
            bounce_ball.related.push(child);
        }
    }
    if bounce_ball.remaining_times > 0 {
        let position = global_transform.translation().xy();
        let dest = vec2(
            rand_range(50.0..CONFIG.display.half_virtual_size.0) * -position.x.signum(),
            position.y,
        );
        let displace = dest - position;
        commands.spawn((
            ChildOf(target),
            enemy::movements::Displacement {
                curve: |x| x,
                duration: Duration::from_secs_f32(displace.length() / 260.0),
                displace,
                notify: Some(event.entity),
            },
        ));
    }
}

fn bounce_ball_shoot_ball(
    mut q_bounce_ball: Query<(Entity, &GlobalTransform, &mut BounceBall)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, global_transform, mut bounce_ball) in q_bounce_ball.iter_mut() {
        if !bounce_ball.start_bouncing {
            continue;
        }
        if bounce_ball.remaining_times > 0 {
            if bounce_ball.interval.tick(time.delta()).just_finished() {
                let translation =
                    global_transform.translation() + vec3(0.0, 20.0 * -bounce_ball.sign, -0.2);
                commands.spawn((
                    Ball::default(),
                    Transform::from_translation(translation),
                    LinearVelocity(vec2(rand_range(225.0..380.0) * rand_sign(), 0.0)),
                    ConstantLinearAcceleration(vec2(0.0, 240.0 * -bounce_ball.sign)),
                ));
                bounce_ball.remaining_times -= 1;
            }
        } else if bounce_ball.wait.tick(time.delta()).just_finished() {
            if rand_bool(0.35) {
                commands.trigger(BehaveQueue::new(entity, "Ruler_Swipe"));
            } else if rand_bool(0.4) {
                commands.trigger(BehaveQueue::new(entity, "Ruler_LaserAttack"));
            }
            bounce_ball.start_bouncing = false;
            for child in bounce_ball.related.drain(..) {
                commands.entity(child).despawn();
            }
            commands.trigger(BehaveEnd {
                entity,
                cooldown: Duration::from_secs_f32(0.3),
                occupies: occupies![("BounceBall", rand_normal(9.0, 0.5) + rand_sign() * 2.0)],
            });
        }
    }
}
