use super::*;

pub(super) struct Phase1Plugin;

impl Plugin for Phase1Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(WCAT_STATE.clone()), |mut commands: Commands| {});
        app.add_systems(OnExit(WCAT_STATE.clone()), |mut commands: Commands| {});
        app.add_systems(
            FixedPostUpdate,
            (high_lunge_wait, high_lunge_end)
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
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct WcatPhase1;

#[derive(Component, Debug, Clone)]
#[require(Behavior::new("Wcat_HighLunge", 1.0, ["Animation", "Velocity"]), BaseFartherBetter::new(0.0, 200.0))]
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
#[require(movements::PartialVelocity::unlinked())]
struct HighLungeTimers {
    waiting: Timer,
    parameters: HighLunge,
    jumping: Timer,
    source: Entity,
}

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
        },
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
                cooldown: Duration::from_secs_f32(rand_normal(1.5, 0.5).clamp(0.5, 2.0)),
                occupies: occupies![],
            })
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
