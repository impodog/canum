use super::*;
use enemy::behavior::*;

pub(super) struct RobotPlugin;

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (dash_away_multiplier,)
                .in_set(BehaviorProcess::Calc)
                .in_set(FactorySystems),
        );
        app.add_systems(
            FixedUpdate,
            (rotate_robot_saw, (end_saw_wipe, saw_wipe).chain()).in_set(FactorySystems),
        );
        app.add_systems(
            FixedUpdate,
            (
                initialize_conveyor,
                conveyor_work,
                conveyor_accelerate,
                update_conveyor_sweep,
            )
                .chain()
                .in_set(FactorySystems),
        );
        app.world_mut()
            .register_component_hooks::<RobotBehaviors>()
            .on_add(robot_behaviors_hook);
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct RobotBehaviors;

fn robot_behaviors_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    // world
    //     .commands()
    //     .spawn((ChildOf(entity), DashAway::default()))
    //     .observe(start_dash_away)
    //     .observe(end_dash_away);
    // world
    //     .commands()
    //     .spawn((ChildOf(entity), SawWipe::default()))
    //     .observe(start_saw_wipe);
    world
        .commands()
        .spawn((ChildOf(entity), ConveyorSweep::default()))
        .observe(start_conveyor_sweep)
        .observe(spawn_conveyor_sweep)
        .observe(intervene_conveyor_sweep);
}

#[derive(Component, Default)]
#[require(Behavior::new("Factory_Robot_DashAway", 0.35, ["Move", "DashAway"]), MultiplierManual, MultiplierWhenResourceOccupied::new([("Main", 0.0)]))]
pub struct DashAway {
    target: Option<Entity>,
}

fn dash_away_multiplier(
    mut q_dash_away: Query<(&mut MultiplierManual, &GlobalTransform), With<DashAway>>,
    q_projectile: Query<(&GlobalTransform, &health::ContactDamage), With<health::PlayerRelated>>,
    attack_info: Res<player::attack::PlayerAttackInfo>,
) {
    // Only update occasionally.
    if rand_bool(0.8) {
        return;
    }
    let (position, damage) = q_projectile.iter().fold((Vec2::ZERO, 0.0f32), |prev, new| {
        let value = new.1.value as f32 * 0.01;
        (prev.0 + new.0.translation().xy() * value, prev.1 + value)
    });
    let average = position * damage.recip().min(1e6);
    for (mut multiplier, global_transform) in q_dash_away.iter_mut() {
        let position = global_transform.translation().xy();
        let distance = average.distance(position);
        let weight = ((distance + 50.0).recip() * 50.0).clamp(0.0, 1.0);
        let weight = (weight + attack_info.charge_ratio).clamp(0.0, 1.0);
        multiplier.0 = weight;
    }
}

fn start_dash_away(
    event: On<BehaveStart>,
    mut commands: Commands,
    mut q_dash_away: Query<&mut DashAway>,
    player: If<Res<player::PrimaryPlayer>>,
    q_transform: Query<&GlobalTransform>,
) {
    let Ok(mut dash_away) = q_dash_away.get_mut(event.entity) else {
        return;
    };
    dash_away.target = Some(event.target);
    let Ok(transform) = q_transform.get(event.entity) else {
        return;
    };
    let Ok(player_transform) = q_transform.get(player.entity()) else {
        return;
    };
    commands.trigger(canum_tool::animation::SpriteSheetQueueUpdate::new(
        event.target,
        10,
        "Factory_Robot_Dash",
    ));
    let position = transform.translation().xy();
    let player_position = player_transform.translation().xy();
    let diff = (position - player_position).normalize_or(vec2(1.0, 0.0));
    let distance = rand_range(80.0f32..120.0);
    let angle = std::f32::consts::FRAC_PI_2 * rand_range(0.9..1.1) * rand_sign();
    let displace = diff.rotate(Vec2::from_angle(angle)) * distance;
    commands.spawn((
        ChildOf(event.target),
        enemy::movements::Displacement {
            curve: canum_fx::quadratic_curve!(1.0, 0.2),
            displace,
            duration: Duration::from_secs_f32(distance / 550.0),
            notify: Some(event.entity),
        },
    ));
    commands.spawn(Sound::new("Bread_Dash").with_volume_add(-5.0));
}

fn end_dash_away(
    event: On<enemy::movements::DisplacementComplete>,
    q_dash_away: Query<&DashAway>,
    mut commands: Commands,
) {
    let Ok(dash_away) = q_dash_away.get(event.entity) else {
        return;
    };
    let Some(target) = dash_away.target else {
        return;
    };
    commands.trigger(canum_tool::animation::SpriteSheetQueueUpdate::new(
        target,
        10,
        "Factory_Robot_Dash",
    ));
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(rand_range(0.1..0.15)),
        occupies: occupies![
            ("Move", rand_range(2.0..3.0)),
            ("DashAway", rand_range(2.5..4.0))
        ],
    });
}

#[derive(Component, Default)]
#[require(
    SessionOnly,
    SpriteSheet::new("Factory_Robot_Saw"),
    canum_tool::animation::SpriteSheetPlayer,
    RigidBody::Kinematic,
    Collider::circle(30.0),
    Mass(5.0),
    LockedAxes::ROTATION_LOCKED,
    Restitution::new(0.5),
    health::Friendly(false),
    health::ContactDamage { value: consts::damage::ONE_STRONG, projectile: false, order: consts::order::ENEMY_BOSS },
    projectile::NoCollideBoundary,
    canum_fx::emphasis::LeaveTrail::new_enabled(0.25, 0.12),
)]
pub struct RobotSaw;

#[derive(Component, Default)]
#[require(Behavior::new("Factory_Robot_SawWipe", 0.6, ["Main", "SawWipe"]))]
struct SawWipe {
    target: Option<Entity>,
    saw: Option<Entity>,
    velocity: Option<Entity>,
    wait: Timer,
    timer: Timer,
    resting: Timer,
}

#[derive(Component)]
struct SawWipeVelocity;

fn start_saw_wipe(
    event: On<BehaveStart>,
    mut commands: Commands,
    mut q_saw_wipe: Query<&mut SawWipe>,
) {
    let Ok(mut saw_wipe) = q_saw_wipe.get_mut(event.entity) else {
        return;
    };
    saw_wipe.target = Some(event.target);
    let saw_entity = commands
        .spawn((
            RobotSaw,
            canum_fx::transform::Follow::new(event.target),
            Transform::from_translation(vec3(64.0, 0.0, -0.1)),
        ))
        .id();
    saw_wipe.saw = Some(saw_entity);
    commands.spawn((
        Sound::new("Factory_Saw"),
        canum_fx::util::DespawnCheck::new(saw_entity),
    ));
    if saw_wipe.velocity.is_none() {
        let velocity_entity = commands
            .spawn((
                ChildOf(event.target),
                movements::PartialVelocity::linked(event.entity),
                SawWipeVelocity,
            ))
            .id();
        saw_wipe.velocity = Some(velocity_entity);
    }
    saw_wipe.wait = Timer::from_seconds(0.25, TimerMode::Once);
    saw_wipe.timer = Timer::from_seconds(rand_normal(9.0, 1.0).max(7.0), TimerMode::Once);
}

fn end_saw_wipe(
    mut q_saw_wipe: Query<(Entity, &mut SawWipe)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut saw_wipe) in q_saw_wipe.iter_mut() {
        if saw_wipe.timer.tick(time.delta()).just_finished() {
            saw_wipe.target = None;
            if let Some(saw) = std::mem::take(&mut saw_wipe.saw) {
                commands.entity(saw).try_despawn();
            }
            commands.trigger(BehaveEnd {
                entity,
                cooldown: Duration::from_secs_f32(0.8),
                occupies: occupies![
                    ("SawWipe", rand_normal(8.0, 1.0)),
                    ("Main", rand_normal(0.5, 0.2).clamp(0.1, 1.0))
                ],
            });
        }
    }
}

fn saw_wipe(
    mut q_saw_wipe: Query<&mut SawWipe>,
    mut q_velocity: Query<&mut movements::PartialVelocity, With<SawWipeVelocity>>,
    q_linear_velocity: Query<&LinearVelocity>,
    q_no_displacement: Query<(), Without<enemy::movements::Displacement>>,
    q_transform: Query<&GlobalTransform>,
    player: If<Res<player::RandomPlayer>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let Ok(player_transform) = q_transform.get(player.entity()) else {
        return;
    };
    let player_position = player_transform.translation().xy();
    for mut saw_wipe in q_saw_wipe.iter_mut() {
        let Some(velocity_entity) = saw_wipe.velocity else {
            continue;
        };
        let Ok(mut velocity) = q_velocity.get_mut(velocity_entity) else {
            continue;
        };
        if saw_wipe.timer.just_finished() {
            **velocity = Vec2::ZERO;
            commands
                .entity(velocity_entity)
                .remove::<enemy::movements::Displacement>();
            continue;
        } else if saw_wipe.timer.is_finished() {
            continue;
        } else if !saw_wipe.wait.is_finished() {
            saw_wipe.wait.tick(time.delta());
            continue;
        }
        if !q_no_displacement.get(velocity_entity).is_ok() {
            continue;
        }
        if !saw_wipe.resting.is_finished() {
            saw_wipe.resting.tick(time.delta());
            continue;
        }
        let Some(target_entity) = saw_wipe.target else {
            continue;
        };
        let Ok(transform) = q_transform.get(target_entity) else {
            continue;
        };
        let saw_position = transform.translation().xy();
        let Ok(player_velocity) = q_linear_velocity.get(player.entity()) else {
            return;
        };
        let diff =
            player_position - saw_position + **player_velocity + rand_normal_clamp(0.1, 0.05, 2.0);
        let diff = diff.clamp_length(0.0, 150.0);
        let displace = if rand_bool(0.5) {
            random_side(diff)
        } else {
            canum_fx::math::long_side_relative_to_screen(diff)
        } * rand_normal(1.0, 0.035);
        saw_wipe.resting = Timer::from_seconds(
            displace.length() / 600.0 + rand_normal_clamp(0.06, 0.01, 2.0),
            TimerMode::Once,
        );
        commands
            .entity(velocity_entity)
            .insert(enemy::movements::Displacement {
                curve: canum_fx::cubic_curve!(1.5, 0.8),
                displace,
                duration: Duration::from_secs_f32(displace.length() / rand_range(300.0..340.0)),
                notify: Some(velocity_entity),
            });
    }
}

fn rotate_robot_saw(
    mut q_saw: Query<(&mut Transform, &canum_fx::transform::Follow), With<RobotSaw>>,
    q_velocity: Query<&LinearVelocity>,
    time: Res<Time>,
) {
    const ANGULAR_VELOCITY: f32 = 10.0;
    for (mut transform, follow) in q_saw.iter_mut() {
        let Ok(velocity) = q_velocity.get(follow.target) else {
            return;
        };
        let target = velocity.normalize_or(vec2(1.0, 0.0));
        let position = transform.translation.xy();
        let position = position.rotate_towards(target, time.delta_secs() * ANGULAR_VELOCITY);
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

#[derive(Component, Default)]
#[require(
    SpriteSheet::new("Factory_Conveyor"),
    Collider,
    Sensor,
    CollidingEntities,
    LinearVelocity,
    movements::DirectVelocity,
    projectile::RemoveOutOfBounds,
    health::Friendly::UNFRIENDLY
)]
struct Conveyor {
    direction: Vec2,
    colliding: bevy::ecs::entity::EntityEquivalentHashSet<Entity>,
    velocity: bevy::ecs::entity::EntityEquivalentHashMap<Entity, Entity>,
}
impl Conveyor {
    const ACCELERATION: f32 = 520.0;
    const SPEED: f32 = 500.0;
}

#[derive(Component, Default)]
struct ConveyorVelocity;

fn initialize_conveyor(
    mut q_conveyor: Query<(&mut LinearVelocity, &mut Transform, &Conveyor), Added<Conveyor>>,
) {
    q_conveyor
        .par_iter_mut()
        .for_each(|(mut velocity, mut transform, conveyor)| {
            **velocity = conveyor.direction * Conveyor::SPEED;
            transform.rotation = Quat::from_rotation_z(conveyor.direction.to_angle());
        });
}

fn conveyor_work(
    mut q_conveyor: Query<(Entity, &mut Conveyor, &CollidingEntities)>,
    commands: ParallelCommands,
) {
    q_conveyor
        .par_iter_mut()
        .for_each(|(conveyor_entity, mut conveyor, colliding)| {
            let mut pending_add = Vec::new();
            for entity in colliding.difference(&conveyor.colliding).copied() {
                let velocity = commands.command_scope(|mut commands| {
                    commands
                        .spawn((
                            ChildOf(entity),
                            ConveyorVelocity,
                            movements::PartialVelocity::linked(conveyor_entity),
                        ))
                        .id()
                });
                pending_add.push((entity, velocity));
            }
            let mut pending_remove = Vec::new();
            for entity in conveyor.colliding.difference(colliding).copied() {
                pending_remove.push(entity);
            }
            for remove in pending_remove {
                conveyor.colliding.remove(&remove);
                if let Some(velocity) = conveyor.velocity.remove(&remove) {
                    commands.command_scope(|mut commands| commands.entity(velocity).despawn());
                }
            }
            for (entity, velocity) in pending_add {
                conveyor.colliding.insert(entity);
                conveyor.velocity.insert(entity, velocity);
            }
        });
}

fn conveyor_accelerate(
    q_conveyor: Query<&Conveyor>,
    mut q_velocity: Query<&mut movements::PartialVelocity, With<ConveyorVelocity>>,
    time: Res<Time>,
) {
    let diff = Conveyor::ACCELERATION * time.delta_secs();
    for conveyor in q_conveyor.iter() {
        for velocity in conveyor.velocity.values() {
            let Ok(mut velocity) = q_velocity.get_mut(*velocity) else {
                return;
            };
            **velocity += conveyor.direction * diff;
            **velocity = velocity.clamp_length_max(Conveyor::SPEED * 2.0);
        }
    }
}

#[derive(Component, Default)]
#[require(Behavior::new("Factory_Robot_ConveyorSweep", 0.45, ["Main", "Move", "ConveyorSweep"]))]
struct ConveyorSweep {
    target: Option<Entity>,
    lane: Option<usize>,
    conveyors: Vec<Entity>,
    timer: Option<Timer>,
}

/// The screen is split into this many horizontal lanes for `ConveyorSweep`.
const CONVEYOR_LANE_COUNT: usize = 3;

fn conveyor_lane_height() -> f32 {
    CONFIG.display.screen_size.y / CONVEYOR_LANE_COUNT as f32
}

fn conveyor_lane_center(lane: usize) -> f32 {
    let lane_height = conveyor_lane_height();
    -CONFIG.display.screen_size.y * 0.5 + lane_height * (lane as f32 + 0.5)
}

/// Returns the lane containing the given y coordinate, clamped to the screen.
fn conveyor_lane_at(y: f32) -> usize {
    let lane_height = conveyor_lane_height();
    let index = ((y + CONFIG.display.screen_size.y * 0.5) / lane_height).floor();
    index.clamp(0.0, (CONVEYOR_LANE_COUNT - 1) as f32) as usize
}

fn conveyor_size() -> Vec2 {
    vec2(
        CONFIG.display.screen_size.x * 0.35,
        conveyor_lane_height() * 0.6,
    )
}

fn start_conveyor_sweep(
    event: On<BehaveStart>,
    mut commands: Commands,
    mut q_conveyor_sweep: Query<&mut ConveyorSweep>,
    q_transform: Query<&GlobalTransform>,
) {
    let Ok(mut sweep) = q_conveyor_sweep.get_mut(event.entity) else {
        return;
    };
    sweep.target = Some(event.target);
    sweep.timer = None;
    for conveyor in sweep.conveyors.drain(..) {
        commands.entity(conveyor).try_despawn();
    }

    let Ok(transform) = q_transform.get(event.target) else {
        return;
    };
    let position = transform.translation().xy();

    // Quickly move to a lane different from the current one.
    let current_lane = conveyor_lane_at(position.y);
    let lane = (current_lane + rand_range(1..CONVEYOR_LANE_COUNT)) % CONVEYOR_LANE_COUNT;
    sweep.lane = Some(lane);

    let max_x = CONFIG.display.screen_size.x * 0.8;
    let target = rand_range(-max_x..=max_x);
    let displace = vec2(target - position.x, conveyor_lane_center(lane) - position.y);
    commands.spawn((
        ChildOf(event.target),
        enemy::movements::Displacement {
            curve: canum_fx::quadratic_curve!(1.5, 0.2),
            displace,
            duration: Duration::from_secs_f32((displace.length() / 600.0).max(0.1)),
            notify: Some(event.entity),
        },
    ));
}

fn spawn_conveyor_sweep(
    event: On<enemy::movements::DisplacementComplete>,
    mut commands: Commands,
    mut q_conveyor_sweep: Query<&mut ConveyorSweep>,
) {
    let Ok(mut sweep) = q_conveyor_sweep.get_mut(event.entity) else {
        return;
    };
    // The behavior might have been intervened before the move completed.
    let Some(lane) = sweep.lane else {
        return;
    };

    let size = conveyor_size();
    let half_width = CONFIG.display.screen_size.x * 0.5;
    // The two conveyors come from opposite sides.
    //
    let direction = vec2(rand_sign(), 0.0);
    let other_lanes = (0..CONVEYOR_LANE_COUNT).filter(|other| *other != lane);
    for other_lane in other_lanes {
        // Start just outside the screen on the side the conveyor comes from.
        let start_x = -direction.x * (half_width + size.x * 0.5);
        let conveyor = commands
            .spawn((
                SessionOnly,
                Conveyor {
                    direction,
                    ..default()
                },
                Transform::from_translation(vec3(start_x, conveyor_lane_center(other_lane), 1.0)),
                Collider::rectangle(size.x, size.y),
            ))
            .id();
        sweep.conveyors.push(conveyor);
    }

    // Spawn multiple saws to block movement.

    // Let the conveyors fully cross the screen before ending the behavior.
    let travel = CONFIG.display.screen_size.x + size.x;
    let duration = travel / Conveyor::SPEED + 0.2;
    sweep.timer = Some(Timer::from_seconds(duration, TimerMode::Once));
}

fn update_conveyor_sweep(
    mut q_conveyor_sweep: Query<(Entity, &mut ConveyorSweep)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut sweep) in q_conveyor_sweep.iter_mut() {
        let Some(timer) = sweep.timer.as_mut() else {
            continue;
        };
        timer.tick(time.delta());
        if !timer.just_finished() {
            continue;
        }

        sweep.target = None;
        sweep.lane = None;
        sweep.timer = None;
        for conveyor in sweep.conveyors.drain(..) {
            commands.entity(conveyor).try_despawn();
        }
        commands.trigger(BehaveEnd {
            entity,
            cooldown: Duration::from_secs_f32(0.8),
            occupies: occupies![
                ("ConveyorSweep", rand_normal(6.0, 1.0)),
                ("Main", rand_normal(0.5, 0.2).clamp(0.1, 1.0))
            ],
        });
    }
}

fn intervene_conveyor_sweep(
    event: On<BehaveIntervene>,
    mut q_conveyor_sweep: Query<&mut ConveyorSweep>,
    mut commands: Commands,
) {
    let Ok(mut sweep) = q_conveyor_sweep.get_mut(event.entity) else {
        return;
    };
    for conveyor in sweep.conveyors.drain(..) {
        commands.entity(conveyor).try_despawn();
    }
    sweep.target = None;
    sweep.lane = None;
    sweep.timer = None;
}
