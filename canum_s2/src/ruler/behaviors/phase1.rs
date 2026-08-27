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
        app.add_systems(FixedUpdate, swipe_align_with_player.in_set(RulerSet));
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct RulerPhase1;

fn ruler_phase1_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world
        .commands()
        .spawn_batch([(ChildOf(entity), Swipe::default())]);
}

#[derive(Component, Default)]
#[require(Behavior::new("Ruler_Swipe", 0.2, ["Main", "Swipe"]))]
pub struct Swipe {
    pub target: Option<Entity>,
    pub status: SwipeStatus,
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
            swipe.status = SwipeStatus::Align(
                Timer::from_seconds(rand_normal(3.0, 0.3).max(2.7), TimerMode::Once),
                velocity,
            );
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
