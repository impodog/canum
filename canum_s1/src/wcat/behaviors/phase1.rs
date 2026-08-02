use bevy::transform::commands;

use super::*;
use std::sync::{Arc, Mutex};

pub(super) struct Phase1Plugin;

impl Plugin for Phase1Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(WCAT_STATE.clone()), |mut commands: Commands| {});
        app.add_systems(OnExit(WCAT_STATE.clone()), |mut commands: Commands| {});

        app.add_systems(
            FixedPostUpdate,
            (high_lunge_wait, high_lunge_end, high_lunge_update_shadow)
                .chain()
                .run_if(in_state(WCAT_STATE.clone())),
        );
        app.world_mut()
            .register_component_hooks::<HighLunge>()
            .on_insert(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(high_lunge_start)
                    .observe(high_lunge_intervene);
            });

        app.add_systems(
            FixedPostUpdate,
            (plain_lunge_wait, plain_lunge_stagger).run_if(in_state(WCAT_STATE.clone())),
        );
        app.add_systems(
            FixedUpdate,
            (plain_lunge_change_multiplier, stagger_recovery).run_if(in_state(WCAT_STATE.clone())),
        );
        app.world_mut()
            .register_component_hooks::<PlainLunge>()
            .on_insert(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(plain_lunge_start)
                    .observe(plain_lunge_intervene);
            });

        app.add_systems(
            FixedPostUpdate,
            wander_around_end.run_if(in_state(WCAT_STATE.clone())),
        );
        app.add_systems(
            FixedUpdate,
            wander_around_change_multiplier.run_if(in_state(WCAT_STATE.clone())),
        );
        app.world_mut()
            .register_component_hooks::<WanderAround>()
            .on_insert(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(wander_around_start)
                    .observe(wander_around_intervene);
            });
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct WcatPhase1;

#[derive(Component, Debug, Clone)]
#[require(Behavior::new("Wcat_HighLunge", 0.4, ["Animation", "Velocity", "HighLunge"]), BaseFartherBetter::new(60.0, 200.0))]
pub struct HighLunge {
    pub jump_acceleration: f32,
    /// Time per 100 units.
    pub time: f32,
    pub min_time: f32,
    /// How strong the cat will predict the player's movement by velocity.
    pub predict_strength: f32,
}
impl Default for HighLunge {
    fn default() -> Self {
        Self {
            jump_acceleration: -200.0,
            time: 0.3,
            min_time: 0.4,
            predict_strength: 0.25,
        }
    }
}

#[derive(Component, Debug)]
#[require(movements::PartialVelocity::unlinked(), Transform, Visibility)]
struct HighLungeTimers {
    waiting: Timer,
    parameters: HighLunge,
    jumping: Timer,
    source: Entity,
    /// The velocity that the shadow uses.
    ground_velocity: Vec2,
}

/// Marker for Wcat shadow when jumping.
#[derive(Component, Default)]
#[require(canum_fx::visual::Shadow(32.0))]
struct HighLungeShadow;

fn high_lunge_start(
    event: On<BehaveStart>,
    q_high_lunge: Query<&HighLunge>,
    mut q_wcat: Query<&mut Animation>,
    mut commands: Commands,
) {
    let Ok(mut animation) = q_wcat.get_mut(event.target) else {
        return;
    };
    let Ok(parameters) = q_high_lunge.get(event.entity) else {
        return;
    };
    animation.replace("Wcat_HighLunge_Prepare", false, None);
    commands.spawn((
        ChildOf(event.target),
        HighLungeTimers {
            waiting: Timer::from_seconds(rand_normal(2.0, 1.0).clamp(1.0, 3.0), TimerMode::Once),
            jumping: Timer::from_seconds(parameters.min_time, TimerMode::Once),
            parameters: parameters.clone(),
            source: event.entity,
            ground_velocity: default(),
        },
        children![(HighLungeShadow, Visibility::Hidden)],
    ));
}

fn high_lunge_wait(
    mut q_wcat: Query<(&GlobalTransform, &mut Animation)>,
    mut q_timers: Query<(
        &ChildOf,
        &mut HighLungeTimers,
        &mut movements::PartialVelocity,
    )>,
    mut commands: Commands,
    time: Res<Time>,
    q_player: Query<(&GlobalTransform, &LinearVelocity)>,
    primary_player: Option<Res<canum_play::player::PrimaryPlayer>>,
) {
    let Some(primary_player) = primary_player else {
        return;
    };
    for (parent, mut timers, mut partial_velocity) in q_timers.iter_mut() {
        if timers.waiting.is_finished() {
            if !timers.jumping.is_finished() {
                timers.jumping.tick(time.delta());
                partial_velocity.y += timers.parameters.jump_acceleration * time.delta_secs();
            }
        } else if timers.waiting.tick(time.delta()).just_finished() {
            let Ok((wcat_transform, mut animation)) = q_wcat.get_mut(parent.0) else {
                return;
            };
            animation.replace("Wcat_HighLunge_Jump", false, None);
            commands.entity(parent.0).insert(ColliderDisabled);

            let Ok((player_transform, linear_velocity)) = q_player.get(primary_player.0) else {
                continue;
            };
            let wcat_position = wcat_transform.translation().xy();
            let player_position = player_transform.translation().xy();
            let target_position = player_position
                + **linear_velocity
                    * rand_normal(
                        timers.parameters.predict_strength,
                        timers.parameters.predict_strength * 0.5,
                    )
                    .clamp(0.0, timers.parameters.predict_strength);
            let diff = target_position - wcat_position;

            let time =
                (diff.length() / 100.0 * timers.parameters.time).max(timers.parameters.min_time);
            let velocity = Vec2::new(
                diff.x / time,
                diff.y / time - 0.5 * timers.parameters.jump_acceleration * time,
            );
            timers.jumping.set_duration(Duration::from_secs_f32(time));

            timers.ground_velocity = diff * time.recip();

            **partial_velocity = velocity;
        }
    }
}

fn high_lunge_end(
    q_timers: Query<(Entity, &ChildOf, &HighLungeTimers)>,
    q_collider_disabled: Query<(), With<ColliderDisabled>>,
    mut q_animation: Query<&mut Animation>,
    mut commands: Commands,
) {
    for (entity, parent, timers) in q_timers.iter() {
        if timers.jumping.elapsed_secs() >= timers.jumping.duration().as_secs_f32() * 0.9
            && q_collider_disabled.get(parent.0).is_ok()
        {
            commands.entity(parent.0).try_remove::<ColliderDisabled>();
        }
        if timers.jumping.is_finished() {
            if let Ok(mut animation) = q_animation.get_mut(parent.0) {
                animation.replace("Wcat_Static", false, None);
            }
            commands.entity(parent.0).try_remove::<ColliderDisabled>();
            commands.entity(entity).despawn();
            commands.trigger(BehaveEnd {
                entity: timers.source,
                cooldown: Duration::from_secs_f32(rand_normal(2.0, 0.5).clamp(1.5, 2.5)),
                occupies: occupies![("HighLunge", rand::random_range(6.0..7.0))],
            })
        }
    }
}

fn high_lunge_update_shadow(
    mut q_shadow: Query<(&mut Visibility, &mut Transform, &ChildOf), With<HighLungeShadow>>,
    q_parent: Query<(&HighLungeTimers, &movements::PartialVelocity)>,
    time: Res<Time>,
) {
    for (mut visibility, mut transform, parent) in q_shadow.iter_mut() {
        let Ok((timers, partial_velocity)) = q_parent.get(parent.0) else {
            return;
        };
        if timers.waiting.just_finished() {
            *visibility = Visibility::Inherited;
        } else if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
        if timers.waiting.is_finished() {
            let displacement = (timers.ground_velocity - **partial_velocity) * time.delta_secs();
            transform.translation.x += displacement.x;
            transform.translation.y += displacement.y;
        }
    }
}

fn high_lunge_intervene(
    event: On<BehaveIntervene>,
    mut commands: Commands,
    q_children: Query<&Children>,
    q_high_lunge: Query<(), With<HighLungeTimers>>,
) {
    let Ok(children) = q_children.get(event.target) else {
        return;
    };
    commands.entity(event.target).remove::<ColliderDisabled>();
    for child in children.iter() {
        if q_high_lunge.get(child).is_ok() {
            commands.entity(child).despawn();
        }
    }
}

#[derive(Component, Debug, Clone)]
#[require(Behavior::new("Wcat_PlainLunge", 1.0, ["Animation", "Velocity"]), BaseByDistance::new(160.0, 40.0), MultiplierManual)]
pub struct PlainLunge {
    pub time: f32,
    pub min_time: f32,
}
impl Default for PlainLunge {
    fn default() -> Self {
        Self {
            time: 0.25,
            min_time: 0.3,
        }
    }
}

fn plain_lunge_change_multiplier(
    mut q_plain_lunge: Query<(Entity, &mut MultiplierManual, &GlobalTransform), With<PlainLunge>>,
    q_parent: Query<&ChildOf>,
    q_stagger_times: Query<&StaggerTimes>,
    q_transform: Query<&GlobalTransform>,
    primary_player: Option<Res<player::PrimaryPlayer>>,
) {
    let Some(primary_player) = primary_player else {
        return;
    };
    let Ok(player_transform) = q_transform.get(primary_player.0) else {
        return;
    };
    let player_position = player_transform.translation().xy();
    for (entity, mut multiplier, global_transform) in q_plain_lunge.iter_mut() {
        let source_position = global_transform.translation().xy();
        if let Some(direction) = (player_position - source_position).try_normalize() {
            // The farther Wcat and player's connecting line is to the center point, the bigger the multiplier is.
            let distance =
                (direction.x * source_position.x - direction.y * source_position.y).abs();
            multiplier.0 = CubicInCurve.sample(distance / 110.0).unwrap_or(1.0);
        } else {
            multiplier.0 = 0.0;
        }
        if let Ok(stagger_times) = q_parent
            .get(entity)
            .and_then(|manager| q_parent.get(manager.0))
            .and_then(|wcat| q_stagger_times.get(wcat.0))
        {
            if stagger_times.0 >= 2 {
                multiplier.0 *= 0.75;
            } else if stagger_times.0 >= 4 {
                multiplier.0 *= 0.5;
            } else if stagger_times.0 >= 6 {
                multiplier.0 *= 0.18
            }
        }
    }
}

#[derive(Component)]
#[require(movements::PartialVelocity::unlinked())]
struct PlainLungeTimers {
    waiting: Timer,
    parameters: PlainLunge,
    source: Entity,
}

fn plain_lunge_start(
    event: On<BehaveStart>,
    q_plain_lunge: Query<&PlainLunge>,
    mut q_wcat: Query<&mut Animation>,
    mut commands: Commands,
) {
    let Ok(mut animation) = q_wcat.get_mut(event.target) else {
        return;
    };
    let Ok(parameters) = q_plain_lunge.get(event.entity) else {
        return;
    };
    animation.replace("Wcat_PlainLunge_Prepare", false, None);
    commands.spawn((
        ChildOf(event.target),
        PlainLungeTimers {
            waiting: Timer::from_seconds(rand_normal(1.5, 0.3).clamp(1.0, 2.0), TimerMode::Once),
            parameters: parameters.clone(),
            source: event.entity,
        },
    ));
}

fn plain_lunge_wait(
    mut q_wcat: Query<(&GlobalTransform, &mut Animation)>,
    mut q_timers: Query<(Entity, &ChildOf, &mut PlainLungeTimers)>,
    mut commands: Commands,
    time: Res<Time>,
    q_player: Query<(&GlobalTransform, &LinearVelocity)>,
    primary_player: Option<Res<canum_play::player::PrimaryPlayer>>,
) {
    let Some(primary_player) = primary_player else {
        return;
    };
    for (entity, parent, mut timers) in q_timers.iter_mut() {
        if timers.waiting.is_finished() {
            // do nothing
        } else if timers.waiting.tick(time.delta()).just_finished() {
            let Ok((wcat_transform, mut animation)) = q_wcat.get_mut(parent.0) else {
                return;
            };
            animation.replace("Wcat_PlainLunge_Jump", false, None);

            let Ok((player_transform, linear_velocity)) = q_player.get(primary_player.0) else {
                continue;
            };
            let wcat_position = wcat_transform.translation().xy();
            let player_position = player_transform.translation().xy();
            let target_position =
                player_position + **linear_velocity * rand_normal(0.25, 0.1).clamp(0.0, 0.5);
            let displace = target_position - wcat_position;

            let time = (displace.length() / 100.0 * timers.parameters.time)
                .max(timers.parameters.min_time);

            commands.entity(entity).observe(plain_lunge_end).insert(
                enemy::movements::Displacement {
                    curve: move |x| {
                        const ACCELERATE_LENGTH: f32 = 0.1;
                        if x < ACCELERATE_LENGTH {
                            CircularInCurve.sample(x / ACCELERATE_LENGTH).unwrap()
                                * ACCELERATE_LENGTH
                        } else if x > 1.0 - ACCELERATE_LENGTH {
                            1.0 - CircularInCurve
                                .sample((1.0 - x) / ACCELERATE_LENGTH)
                                .unwrap()
                                * ACCELERATE_LENGTH
                        } else {
                            x
                        }
                    },
                    displace,
                    duration: Duration::from_secs_f32(time),
                    notify: Some(entity),
                },
            );
        }
    }
}

fn plain_lunge_end(
    event: On<enemy::movements::DisplacementComplete>,
    mut commands: Commands,
    q_timers: Query<(&ChildOf, &PlainLungeTimers)>,
    mut q_animation: Query<&mut Animation>,
) {
    let Ok((parent, timers)) = q_timers.get(event.entity) else {
        return;
    };
    if let Ok(mut animation) = q_animation.get_mut(parent.0) {
        animation.replace("Wcat_Static", false, None);
    }
    commands.trigger(BehaveEnd {
        entity: timers.source,
        cooldown: Duration::from_secs_f32(rand::random_range(1.0..2.0)),
        occupies: occupies![],
    });
    commands.entity(event.entity).despawn();
}

fn plain_lunge_intervene(
    event: On<BehaveIntervene>,
    mut commands: Commands,
    q_children: Query<&Children>,
    q_plain_lunge: Query<(), With<PlainLungeTimers>>,
) {
    let Ok(children) = q_children.get(event.target) else {
        return;
    };
    for child in children.iter() {
        if q_plain_lunge.get(child).is_ok() {
            commands.entity(child).despawn();
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct StaggerRecovery {
    pub timer: Timer,
}
impl Default for StaggerRecovery {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(rand::random_range(4.5..6.0), TimerMode::Once),
        }
    }
}

fn plain_lunge_stagger(
    q_timers: Query<(
        &ChildOf,
        &PlainLungeTimers,
        &enemy::movements::DisplacementPercentage,
    )>,
    q_stagger: Query<(Entity, &GlobalTransform), With<super::bar::CanStaggerWcat>>,
    mut q_wcat: Query<(
        &mut StaggerTimes,
        &LinearVelocity,
        &GlobalTransform,
        &Children,
    )>,
    q_manager: Query<(), With<BehaviorManager>>,
    mut q_animation: Query<&mut Animation>,
    collisions: Collisions,
    mut commands: Commands,
) {
    for (parent, timers, percentage) in q_timers.iter() {
        let Ok((mut stagger_times, linear_velocity, transform, children)) =
            q_wcat.get_mut(parent.0)
        else {
            return;
        };
        if timers.waiting.is_finished()
            && (0.1..=0.92f32).contains(&**percentage)
            && q_stagger.iter().any(|(stagger, stagger_transform)| {
                // Colliding, the bump angle is almost vertical, and must be bump into instead of bump out of.
                collisions.contains(parent.0, stagger)
                    && (stagger_transform.rotation().to_euler(EulerRot::XYZ).2
                        - linear_velocity.to_angle())
                    .cos()
                    .abs()
                        <= 0.7
                    && (stagger_transform.translation().xy() - transform.translation().xy())
                        .dot(**linear_velocity)
                        > 0.0
            })
        {
            stagger_times.0 += 1;
            let manager = children.iter().find(|child| q_manager.get(*child).is_ok());
            if let Some(manager) = manager {
                commands.trigger(BehaveIntervene::new_for_manager(manager));
            } else {
                warn!("Unable to find manager for Wcat entity");
            }
            commands.entity(parent.0).insert(StaggerRecovery::default());
            commands.spawn(Sound::new("Wcat_Bump"));
            let Ok(mut animation) = q_animation.get_mut(parent.0) else {
                return;
            };
            animation.replace("Wcat_Staggered", false, None);
        }
    }
}

fn stagger_recovery(
    mut q_recovery: Query<(Entity, &mut StaggerRecovery, &Children)>,
    mut q_manager: Query<&mut BehaviorManager>,
    mut q_animation: Query<&mut Animation>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut recovery, children) in q_recovery.iter_mut() {
        if recovery.timer.tick(time.delta()).just_finished() {
            for child in children.iter() {
                if let Ok(mut manager) = q_manager.get_mut(child) {
                    manager.disabled = false;
                }
            }
            if let Ok(mut animation) = q_animation.get_mut(entity) {
                animation.replace("Wcat_Static", false, None);
            }
            commands.entity(entity).remove::<StaggerRecovery>();
        }
    }
}

#[derive(Component, Default, Clone)]
#[require(Behavior::new("Wcat_WanderAround", 1.0, ["WanderAround"]), MultiplierManual, MultiplierWhenResourceOccupied::new([("Velocity", 0.5)]))]
pub struct WanderAround;

#[derive(Component)]
#[require(movements::PartialVelocity::unlinked())]
pub struct WanderTimer {
    pub timer: Timer,
    pub source: Entity,
}

fn wander_around_start(
    event: On<BehaveStart>,
    mut commands: Commands,
    q_transform: Query<&GlobalTransform>,
) {
    let Ok(transform) = q_transform.get(event.target) else {
        return;
    };
    let direction = transform.translation().xy().normalize_or(vec2(1.0, 0.0));
    let direction = direction.rotate(Vec2::from_angle(rand_normal(
        0.0,
        std::f32::consts::FRAC_PI_3,
    )));
    commands.spawn((
        ChildOf(event.target),
        WanderTimer {
            timer: Timer::from_seconds(rand_normal(0.8, 0.3).clamp(0.2, 1.0), TimerMode::Once),
            source: event.entity,
        },
        movements::PartialVelocity {
            velocity: direction * 70.0,
            linked: None,
        },
    ));
}

#[allow(clippy::collapsible_if)]
fn wander_around_end(
    mut q_wander: Query<(Entity, &ChildOf, &mut WanderTimer)>,
    mut q_animation: Query<&mut Animation>,
    q_children: Query<&Children>,
    q_manager: Query<(&BehaviorManager, &BehaviorManagerInfo)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, parent, mut timer) in q_wander.iter_mut() {
        if timer.timer.tick(time.delta()).just_finished() {
            if let Ok((manager, manager_info)) = q_children.get(parent.0).and_then(|children| {
                children
                    .iter()
                    .find_map(|child| {
                        let result = q_manager.get(child);
                        if result.is_ok() { Some(result) } else { None }
                    })
                    .unwrap_or(Result::Err(
                        // Just placeholder; Never used.
                        bevy::ecs::query::QueryEntityError::AliasedMutability(entity),
                    ))
            }) {
                if !manager_info.occupied().contains_key("Animation") && !manager.disabled {
                    let Ok(mut animation) = q_animation.get_mut(parent.0) else {
                        return;
                    };
                    animation.replace("Wcat_Static", false, None);
                }
            }
            commands.trigger(BehaveEnd {
                entity: timer.source,
                cooldown: Duration::default(),
                occupies: occupies![("WanderAround", 0.2)],
            });
            commands.entity(entity).despawn();
        } else {
            let Ok(mut animation) = q_animation.get_mut(parent.0) else {
                return;
            };
            if animation.name == "Wcat_Static" {
                animation.replace("Wcat_Walk", false, None);
            }
        }
    }
}

fn wander_around_intervene(
    event: On<BehaveIntervene>,
    q_children: Query<&Children>,
    q_wander: Query<(), With<WanderAround>>,
    mut commands: Commands,
) {
    let Ok(children) = q_children.get(event.target) else {
        return;
    };
    for child in children.iter() {
        if q_wander.get(child).is_ok() {
            commands.entity(child).despawn();
        }
    }
}

fn wander_around_change_multiplier(
    mut q_wander_around: Query<(&mut MultiplierManual, &GlobalTransform), With<WanderAround>>,
) {
    for (mut multiplier, transform) in q_wander_around.iter_mut() {
        // The cat will want to go away from the bar in the middle.
        let position = transform.translation().xy();
        let distance = position.length();
        if distance <= 100.0 {
            multiplier.0 = 2.0;
        } else if distance <= 135.0 {
            multiplier.0 = 1.5;
        } else if distance <= 200.0 {
            multiplier.0 = 1.2;
        }
    }
}
