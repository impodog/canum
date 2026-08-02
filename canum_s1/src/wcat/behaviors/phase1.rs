use super::*;

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
            plain_lunge_wait.run_if(in_state(WCAT_STATE.clone())),
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
    animation.replace("Wcat_HighLunge_Prepare", true, None);
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
            animation.replace("Wcat_HighLunge_Jump", true, None);
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
    mut commands: Commands,
) {
    for (entity, parent, timers) in q_timers.iter() {
        if timers.jumping.elapsed_secs() >= timers.jumping.duration().as_secs_f32() * 0.9
            && q_collider_disabled.get(parent.0).is_ok()
        {
            commands.entity(parent.0).try_remove::<ColliderDisabled>();
        }
        if timers.jumping.is_finished() {
            commands.entity(parent.0).try_remove::<ColliderDisabled>();
            commands.entity(entity).despawn();
            commands.trigger(BehaveEnd {
                entity: timers.source,
                cooldown: Duration::from_secs_f32(rand_normal(2.0, 0.5).clamp(1.5, 2.5)),
                occupies: occupies![("HighLunge", 7.5)],
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
#[require(Behavior::new("Wcat_PlainLunge", 1.0, ["Animation", "Velocity"]), BaseByDistance::new(160.0, 40.0))]
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
    animation.replace("Wcat_PlainLunge_Prepare", true, None);
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
            animation.replace("Wcat_PlainLunge_Jump", true, None);

            let Ok((player_transform, linear_velocity)) = q_player.get(primary_player.0) else {
                continue;
            };
            let wcat_position = wcat_transform.translation().xy();
            let player_position = player_transform.translation().xy();
            let target_position =
                player_position + **linear_velocity * rand_normal(0.5, 0.25).clamp(0.0, 1.0);
            let displace = target_position - wcat_position;

            let time = (displace.length() / 100.0 * timers.parameters.time)
                .max(timers.parameters.min_time);

            commands.entity(entity).observe(plain_lunge_end).insert(
                enemy::movements::Displacement {
                    curve: |x| {
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
    q_timers: Query<&PlainLungeTimers>,
) {
    let Ok(timers) = q_timers.get(event.entity) else {
        return;
    };
    commands.trigger(BehaveEnd {
        entity: timers.source,
        cooldown: Duration::from_secs_f32(rand::random_range(0.5..1.5)),
        occupies: occupies![],
    });
    commands.entity(event.entity).despawn();
}

fn plain_lunge_intervene(
    event: On<BehaveIntervene>,
    mut commands: Commands,
    q_children: Query<&Children>,
    q_high_lunge: Query<(), With<PlainLungeTimers>>,
) {
    let Ok(children) = q_children.get(event.target) else {
        return;
    };
    for child in children.iter() {
        if q_high_lunge.get(child).is_ok() {
            commands.entity(child).despawn();
        }
    }
}
