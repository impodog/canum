use super::*;
use enemy::behavior::*;

pub(super) struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(BREAD_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(add_bread_observers)));
        });

        app.world_mut()
            .register_component_hooks::<BumpAround>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(bump_around_start);
            });

        app.world_mut()
            .register_component_hooks::<SpeedyDash>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(speedy_dash_start)
                    .observe(speedy_dash_end);
            });

        app.world_mut()
            .register_component_hooks::<RandomShoot>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(random_shoot_start);
            });

        app.world_mut()
            .register_component_hooks::<SimplyWander>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(simply_wander_start);
            });

        app.world_mut()
            .register_component_hooks::<StreamSlash>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(stream_slash_start)
                    .observe(stream_slash_terminals);
            });

        app.world_mut()
            .register_component_hooks::<SuperRandomShoot>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(super_random_shoot);
            });

        app.add_systems(
            FixedUpdate,
            (
                bump_around_work,
                simply_wander_work,
                stream_slash_shoot,
                bullet_burst,
            )
                .run_if(in_state(BREAD_STATE.clone())),
        );
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct BreadBehaviors;

fn add_bread_observers(
    _event: On<entry::BreadStart>,
    mut commands: Commands,
    q_bread: Query<Entity, With<BreadBoss>>,
) {
    for entity in q_bread.iter() {
        commands.entity(entity).observe(update_stage);
    }
}

fn update_stage(
    event: On<health::Damage>,
    mut q_bread: Query<(&mut BreadBoss, &enemy::health::EnemyHealth)>,
    mut commands: Commands,
) {
    let Ok((mut bread, health)) = q_bread.get_mut(event.entity) else {
        return;
    };
    let stage: u8 = if health.value > 3400 {
        1
    } else if health.value > 2400 {
        2
    } else if health.value > 1000 {
        3
    } else {
        4
    };
    if stage != bread.stage {
        bread.stage = stage;
        commands.trigger(BreadNextStage {
            entity: event.entity,
            stage,
        });
    }
}

#[derive(Component, Default)]
#[require(Behavior::new("Bread_BumpAround", 1.0, ["Stage1", "BumpAround"]))]
pub struct BumpAround {
    velocity: Option<Entity>,
    target: Option<Entity>,
    timer: Timer,
    x_cooldown: Timer,
    y_cooldown: Timer,
    next_shoot: Timer,
}

fn bump_around_start(
    event: On<BehaveStart>,
    mut q_bump_around: Query<&mut BumpAround>,
    mut q_velocity: Query<&mut movements::PartialVelocity>,
    mut commands: Commands,
) {
    let Ok(mut bump_around) = q_bump_around.get_mut(event.entity) else {
        return;
    };
    bump_around.timer = Timer::from_seconds(rand::random_range(7.0..12.0), TimerMode::Once);
    bump_around.target = Some(event.target);
    bump_around.next_shoot = Timer::from_seconds(1.0, TimerMode::Once);
    let velocity = vec2(rand_normal(200.0, 20.0).clamp(170.0, 230.0), 0.0).rotate(
        Vec2::from_angle(rand::random_range(0.0..std::f32::consts::TAU)),
    );
    if let Some(velocity_entity) = bump_around.velocity {
        let Ok(mut partial_velocity) = q_velocity.get_mut(velocity_entity) else {
            return;
        };
        **partial_velocity = velocity;
    } else {
        let velocity_entity = commands
            .spawn((
                ChildOf(event.target),
                movements::PartialVelocity::linked(event.entity).with_velocity(velocity),
            ))
            .id();
        bump_around.velocity = Some(velocity_entity);
    }
}

fn bump_around_work(
    mut q_bump_around: Query<(Entity, &GlobalTransform, &mut BumpAround)>,
    mut q_velocity: Query<&mut movements::PartialVelocity>,
    q_colliding: Query<&CollidingEntities>,
    q_boundary: Query<&GlobalTransform, With<setup::Boundaries>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, global_transform, mut bump_around) in q_bump_around.iter_mut() {
        if bump_around.timer.is_finished() {
            continue;
        }

        let Some((target, velocity_entity)) = bump_around.target.zip(bump_around.velocity) else {
            continue;
        };
        let Ok(mut velocity) = q_velocity.get_mut(velocity_entity) else {
            continue;
        };
        let Ok(colliding) = q_colliding.get(target) else {
            continue;
        };

        if bump_around.timer.tick(time.delta()).just_finished() {
            **velocity = Vec2::ZERO;
            commands.trigger(BehaveEnd {
                entity,
                cooldown: Duration::from_secs_f32(rand_normal(1.5, 0.1)),
                occupies: occupies![(
                    "BumpAround",
                    if rand::random_bool(0.25) { 1.0 } else { 2.0 }
                )],
            });
            continue;
        }
        bump_around.x_cooldown.tick(time.delta());
        bump_around.y_cooldown.tick(time.delta());

        if bump_around.next_shoot.tick(time.delta()).just_finished() {
            const DIFF: f32 = std::f32::consts::TAU / 9.0;
            let start_angle = rand::random_range(0.0..std::f32::consts::PI);
            let mut angle = start_angle;
            let mut flag = true;
            loop {
                let mut slice = projectiles::BreadSlice::default()
                    .with_angle(angle)
                    .with_accelerate(0.5);
                if flag {
                    slice.play_sound = true;
                    flag = false;
                }
                commands.spawn((
                    Transform::from_translation(
                        global_transform.translation() + vec3(0.0, 0.0, -0.1),
                    ),
                    slice,
                ));
                angle = canum_fx::math::normalize_angle(angle + DIFF);
                if (angle - start_angle).abs() < 1e-5 {
                    break;
                }
            }
            bump_around.next_shoot = Timer::from_seconds(rand_normal(2.5, 0.3), TimerMode::Once);
        }

        let mut bumped = false;
        for entity in colliding.iter() {
            if let Ok(global_transform) = q_boundary.get(*entity) {
                let position = global_transform.translation();
                if position.x == 0.0 {
                    if bump_around.y_cooldown.is_finished() {
                        velocity.y = -velocity.y;
                        bump_around.y_cooldown = Timer::from_seconds(0.1, TimerMode::Once);
                        bumped = true;
                    }
                } else {
                    if bump_around.x_cooldown.is_finished() {
                        velocity.x = -velocity.x;
                        bump_around.x_cooldown = Timer::from_seconds(0.1, TimerMode::Once);
                        bumped = true;
                    }
                }
            }
        }
        if bumped {
            **velocity = velocity.rotate(Vec2::from_angle(rand_normal(0.0, 0.15)));
        }
    }
}

#[derive(Component, Default)]
#[require(Behavior::new("Bread_SpeedyDash", 1.0, ["Stage1", "Stage3", "SpeedyDash"]))]
pub struct SpeedyDash {
    direction: Vec2,
}

fn speedy_dash_start(
    event: On<BehaveStart>,
    mut commands: Commands,
    mut q_speedy_dash: Query<&mut SpeedyDash>,
    player: Option<Res<player::PrimaryPlayer>>,
    q_transform: Query<&GlobalTransform>,
) {
    use bevy::math::FloatPow;
    let Some(player) = player else {
        return;
    };
    let Ok(player_transform) = q_transform.get(player.0) else {
        return;
    };
    let player_position = player_transform.translation().xy();
    let Ok(bread_transform) = q_transform.get(event.entity) else {
        return;
    };
    let bread_position = bread_transform.translation().xy();
    let displace = (player_position - bread_position) * 1.1;
    let displace = displace.rotate(Vec2::from_angle(rand_normal(0.0, 0.1)));
    commands.spawn((
        ChildOf(event.target),
        enemy::movements::Displacement {
            curve: |x| (1.125 - (1.5 - x).squared() * 0.5).clamp(0.0, 1.0),
            displace,
            duration: Duration::from_secs_f32((displace.length() / 400.0).clamp(0.3, 1.0)),
            notify: Some(event.entity),
        },
    ));

    let Ok(mut speedy_dash) = q_speedy_dash.get_mut(event.entity) else {
        return;
    };
    speedy_dash.direction = displace.normalize_or(vec2(1.0, 0.0));

    commands.spawn(Sound::new("Bread_Dash"));
}

fn speedy_dash_end(
    event: On<enemy::movements::DisplacementComplete>,
    mut commands: Commands,
    q_speedy_dash: Query<(&GlobalTransform, &SpeedyDash)>,
    player: Option<Res<player::PrimaryPlayer>>,
    q_transform: Query<&GlobalTransform>,
) {
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(1.0),
        occupies: occupies![("SpeedyDash", 1.5)],
    });
    let Ok((global_transform, speedy_dash)) = q_speedy_dash.get(event.entity) else {
        return;
    };
    let translation = global_transform.translation() + vec3(0.0, 0.0, -0.1);
    let position = translation.xy();

    let Some(player) = player else {
        return;
    };
    let Ok(player_transform) = q_transform.get(player.0) else {
        return;
    };
    let player_position = player_transform.translation().xy();
    let target_direction = (player_position - position).normalize_or(vec2(1.0, 0.0));
    let sloped_direction = if rand::random_bool(0.8) {
        target_direction.rotate(Vec2::from_angle(rand_normal(0.0, 0.05)))
    } else {
        Vec2::from_angle(
            rand_normal(std::f32::consts::FRAC_PI_6, 0.2).clamp(0.1, std::f32::consts::FRAC_PI_3),
        )
        .rotate(-speedy_dash.direction)
    };
    let sgn = sloped_direction.dot(speedy_dash.direction).signum();
    for direction in [
        sloped_direction,
        speedy_dash.direction * sgn,
        sloped_direction.reflect(speedy_dash.direction.perp()),
    ] {
        commands.spawn((
            Transform::from_translation(translation),
            projectiles::BreadSlice::default()
                .with_rotate(direction)
                .with_accelerate(0.2)
                .with_velocity_multiply(1.1),
        ));
    }
    commands.spawn(Sound::new("Bread_Shoot"));
}

#[derive(Component, Default)]
#[require(Behavior::new("Bread_RandomShoot", 1.0, ["Stage2", "RandomShoot"]))]
pub struct RandomShoot;

fn random_shoot_start(event: On<BehaveStart>, mut commands: Commands) {
    let x = rand::random_range(0.0..CONFIG.display.half_virtual_size.0) * rand_sign();
    let y = rand::random_range(0.0..CONFIG.display.half_virtual_size.1) * rand_sign();
    commands.spawn((
        Transform::from_translation(vec3(x, y, 0.1)),
        projectiles::BreadSlice::default()
            .with_fade(1.1)
            .with_accelerate(0.1)
            .with_play_sound()
            .with_track_player(),
    ));
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(0.2),
        occupies: occupies![("RandomShoot", rand_normal(0.6, 0.05))],
    });
}
#[derive(Component, Default)]
#[require(Behavior::new("Bread_SimplyWander", 1.0, ["SimplyWander"]))]
pub struct SimplyWander {
    velocity: Option<Entity>,
    target: Option<Entity>,
    x_cooldown: Timer,
    y_cooldown: Timer,
}

fn simply_wander_start(
    event: On<BehaveStart>,
    mut q_simply_wander: Query<&mut SimplyWander>,
    mut commands: Commands,
) {
    let Ok(mut simply_wander) = q_simply_wander.get_mut(event.entity) else {
        return;
    };
    simply_wander.target = Some(event.target);
    let velocity = vec2(rand_normal(130.0, 15.0).clamp(110.0, 140.0), 0.0).rotate(
        Vec2::from_angle(rand::random_range(0.0..std::f32::consts::TAU)),
    );
    if simply_wander.velocity.is_none() {
        let velocity_entity = commands
            .spawn((
                ChildOf(event.target),
                movements::PartialVelocity::linked(event.entity).with_velocity(velocity),
            ))
            .id();
        simply_wander.velocity = Some(velocity_entity);
    }
}

fn simply_wander_work(
    mut q_simply_wander: Query<&mut SimplyWander>,
    mut q_velocity: Query<&mut movements::PartialVelocity>,
    q_colliding: Query<&CollidingEntities>,
    q_boundary: Query<&GlobalTransform, With<setup::Boundaries>>,
    time: Res<Time>,
) {
    for mut simply_wander in q_simply_wander.iter_mut() {
        let Some((target, velocity_entity)) = simply_wander.target.zip(simply_wander.velocity)
        else {
            continue;
        };
        let Ok(mut velocity) = q_velocity.get_mut(velocity_entity) else {
            continue;
        };
        let Ok(colliding) = q_colliding.get(target) else {
            continue;
        };

        simply_wander.x_cooldown.tick(time.delta());
        simply_wander.y_cooldown.tick(time.delta());

        let mut bumped = false;
        for entity in colliding.iter() {
            if let Ok(global_transform) = q_boundary.get(*entity) {
                let position = global_transform.translation();
                if position.x == 0.0 {
                    if simply_wander.y_cooldown.is_finished() {
                        velocity.y = -velocity.y;
                        simply_wander.y_cooldown = Timer::from_seconds(0.1, TimerMode::Once);
                        bumped = true;
                    }
                } else {
                    if simply_wander.x_cooldown.is_finished() {
                        velocity.x = -velocity.x;
                        simply_wander.x_cooldown = Timer::from_seconds(0.1, TimerMode::Once);
                        bumped = true;
                    }
                }
            }
        }
        if bumped {
            **velocity = velocity.rotate(Vec2::from_angle(rand_normal(0.0, 0.3)));
        }
    }
}

#[derive(Component)]
#[require(Behavior::new("Bread_StreamSlash", 1.0, ["Stage3", "StreamSlash"]))]
pub struct StreamSlash {
    move_to_corner: bool,
    slashing: bool,
    timer: Timer,
    target: Option<Entity>,
}
impl Default for StreamSlash {
    fn default() -> Self {
        Self {
            move_to_corner: false,
            slashing: false,
            timer: Timer::from_seconds(0.28, TimerMode::Repeating),
            target: None,
        }
    }
}

fn stream_slash_start(
    event: On<BehaveStart>,
    mut q_stream_slash: Query<(&GlobalTransform, &mut StreamSlash)>,
    mut commands: Commands,
) {
    let Ok((global_transform, mut stream_slash)) = q_stream_slash.get_mut(event.entity) else {
        return;
    };
    stream_slash.target = Some(event.target);

    let position = global_transform.translation().xy();
    let target_position = vec2(
        -CONFIG.display.half_virtual_size.0 + BREAD_HALF_LENGTH,
        -CONFIG.display.half_virtual_size.1 + BREAD_HALF_LENGTH,
    );
    stream_slash.move_to_corner = true;
    stream_slash.slashing = false;
    let displace = target_position - position;
    commands.spawn((
        ChildOf(event.target),
        enemy::movements::Displacement {
            curve: |x| QuadraticInOutCurve.sample(x).unwrap_or(0.0),
            displace,
            duration: Duration::from_secs_f32(displace.length() / 300.0),
            notify: Some(event.entity),
        },
    ));
}

fn stream_slash_terminals(
    event: On<enemy::movements::DisplacementComplete>,
    mut q_stream_slash: Query<&mut StreamSlash>,
    mut commands: Commands,
) {
    let Ok(mut stream_slash) = q_stream_slash.get_mut(event.entity) else {
        return;
    };
    let Some(target) = stream_slash.target else {
        return;
    };
    if stream_slash.move_to_corner {
        stream_slash.move_to_corner = false;
        stream_slash.slashing = true;
        commands.spawn((
            ChildOf(target),
            enemy::movements::Displacement {
                curve: |x| x,
                displace: vec2(CONFIG.display.virtual_size.0 as f32 - BREAD_LENGTH, 0.0),
                duration: Duration::from_secs_f32(3.0),
                // Notify again but in the latter branch.
                notify: Some(event.entity),
            },
        ));
    } else {
        stream_slash.slashing = false;
        commands.trigger(BehaveEnd {
            entity: event.entity,
            cooldown: Duration::from_secs_f32(1.0),
            occupies: occupies![(
                "StreamSlash",
                if rand::random_bool(0.75) { 1.5 } else { 0.1 }
            )],
        });
    }
}

fn stream_slash_shoot(
    mut q_stream_slash: Query<(&GlobalTransform, &mut StreamSlash)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (global_transform, mut stream_slash) in q_stream_slash.iter_mut() {
        if stream_slash.slashing && stream_slash.timer.tick(time.delta()).just_finished() {
            let position = global_transform.translation() + vec3(0.0, 0.0, -0.1);
            let slice = projectiles::BreadSlice::default()
                .with_velocity_multiply(1.2)
                .with_angle(std::f32::consts::FRAC_PI_2)
                .with_accelerate(0.25);
            commands.spawn((
                slice.clone().with_play_sound(),
                Transform::from_translation(position),
            ));
            for position in [
                vec3(
                    -CONFIG.display.half_virtual_size.0 + 10.0,
                    -CONFIG.display.half_virtual_size.1 - 10.0,
                    0.1,
                ),
                vec3(
                    CONFIG.display.half_virtual_size.0 - 10.0,
                    -CONFIG.display.half_virtual_size.1 - 10.0,
                    0.1,
                ),
            ] {
                commands.spawn((Transform::from_translation(position), slice.clone()));
            }
        }
    }
}

#[derive(Component, Default)]
#[require(Behavior::new("Bread_SuperRandomShoot", 1.0, ["Stage4", "SuperRandomShoot"]))]
pub struct SuperRandomShoot;

#[derive(Component, Default)]
struct CanBurst;

fn super_random_shoot(event: On<BehaveStart>, mut commands: Commands) {
    let x = rand::random_range(0.0..CONFIG.display.half_virtual_size.0) * rand_sign();
    let y = rand::random_range(0.0..CONFIG.display.half_virtual_size.1) * rand_sign();
    commands.spawn((
        Transform::from_translation(vec3(x, y, 0.1)),
        CanBurst,
        projectiles::BreadSlice::default()
            .with_fade(1.5)
            .with_accelerate(0.1)
            .with_velocity_multiply(1.0)
            .with_play_sound()
            .with_track_player(),
    ));
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(0.2),
        occupies: occupies![("SuperRandomShoot", rand_normal(1.8, 0.15))],
    })
}

fn bullet_burst(
    q_can_burst: Query<(Entity, &GlobalTransform, &LinearVelocity), With<CanBurst>>,
    mut commands: Commands,
) {
    for (entity, transform, linear_velocity) in q_can_burst.iter() {
        let position = transform.translation() + vec3(0.0, 0.0, 0.05);
        if !CONFIG.display.screen_rect.contains(position.xy()) {
            commands.entity(entity).despawn();
            let initial_angle = -linear_velocity.normalize_or(vec2(1.0, 0.0));
            let angle = rand_normal(0.5, 0.16).clamp(0.3, 0.75);
            for angle in [-angle, 0.0, angle] {
                commands.spawn((
                    Transform::from_translation(position),
                    projectiles::BreadSlice::default()
                        .with_rotate(initial_angle)
                        .with_angle(angle)
                        .with_velocity_multiply(1.2),
                ));
            }
            commands.spawn(Sound::new("Bread_Explode"));
        }
    }
}
