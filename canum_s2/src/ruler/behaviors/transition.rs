use super::*;

pub(super) struct TransitionPlugin;

impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<RulerPhase>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(enemy_defeated);
            });
        app.add_systems(OnEnter(RULER_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(spawn_phase1_health_bar)));
            commands.spawn((SessionOnly, Observer::new(spawn_phase2_health_bar)));
            commands.spawn((SessionOnly, Observer::new(enter_phase2_update_ruler)));
        });
        app.add_systems(FixedUpdate, phase2_transition);
    }
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulerPhase {
    #[default]
    Phase1,
    Phase2,
}

/// Marks the health bar for transition phase1 -> phase2 to temporarily despawn it.
#[derive(Component, Default)]
struct RulerHealthBar;

fn spawn_phase1_health_bar(
    _event: On<entry::SpawnHealthBar>,
    save: Res<Save>,
    mut commands: Commands,
    bottom_center: Single<Entity, With<canum_ui::BottomCenter>>,
    ruler: Single<Entity, With<RulerBoss>>,
) {
    if save.progress.selected_effects.contains("ShowHealth") {
        commands.spawn((
            ChildOf(bottom_center.entity()),
            canum_ui::bar::AssociatedBoss(ruler.entity()),
            canum_ui::bar::health_bar(
                Color::Srgba(Srgba::hex("#2eb6ff").unwrap()),
                Color::Srgba(Srgba::hex("#2d065a").unwrap()),
                canum_ui::bar::HealthBar {
                    total: PHASE1_HEALTH as f32,
                    current: PHASE1_HEALTH as f32,
                    ..default()
                },
            ),
            RulerHealthBar,
        ));
    }
}

fn spawn_phase2_health_bar(
    _event: On<EnterPhase2>,
    save: Res<Save>,
    mut commands: Commands,
    bottom_center: Single<Entity, With<canum_ui::BottomCenter>>,
    ruler: Single<Entity, With<RulerBoss>>,
) {
    if save.progress.selected_effects.contains("ShowHealth") {
        commands.spawn((
            ChildOf(bottom_center.entity()),
            canum_ui::bar::AssociatedBoss(ruler.entity()),
            canum_ui::bar::health_bar(
                Color::Srgba(Srgba::hex("#015aff").unwrap()),
                Color::Srgba(Srgba::hex("#2d065a").unwrap()),
                canum_ui::bar::HealthBar {
                    total: PHASE2_HEALTH as f32,
                    current: PHASE2_HEALTH as f32,
                    ..default()
                },
            ),
            RulerHealthBar,
        ));
    }
}

#[derive(EntityEvent)]
pub(super) struct EnterPhase2 {
    entity: Entity,
}

#[allow(clippy::type_complexity)]
fn enemy_defeated(
    event: On<enemy::health::EnemyDefeated>,
    mut q_ruler: Query<(&Children, &mut RulerPhase, &GlobalTransform)>,
    q_child_despawn: Query<
        (),
        Or<(
            With<phase1::RulerPhase1>,
            With<enemy::movements::Displacement>,
        )>,
    >,
    health_bar: Query<Entity, With<RulerHealthBar>>,
    mut commands: Commands,
) {
    let Ok((children, mut phase, global_transform)) = q_ruler.get_mut(event.entity) else {
        return;
    };
    let position = global_transform.translation().xy();
    match *phase {
        RulerPhase::Phase1 => {
            for child in children
                .iter()
                .filter(|child| q_child_despawn.get(*child).is_ok())
            {
                commands.entity(child).despawn();
            }
            for entity in health_bar.iter() {
                commands.entity(entity).despawn();
            }
            *phase = RulerPhase::Phase2;
            let target_position = vec2(
                0.0,
                -CONFIG.display.half_virtual_size.1 + PHASE2_SIZE.y * 0.5,
            );
            commands.spawn((
                ChildOf(event.entity),
                enemy::movements::Displacement {
                    displace: target_position - position,
                    curve: |x| QuadraticInOutCurve.sample(x).unwrap(),
                    duration: TRANSITION_TIME,
                    notify: None,
                },
                canum_fx::physics::ParentColliderDisabled,
            ));
            commands
                .entity(event.entity)
                .insert(TransitionTimer::default());
            commands.trigger(canum_fx::emphasis::LeaveTrailSetting::disable(event.entity));
        }
        RulerPhase::Phase2 => {
            todo!("phase2 cinematic defeat animation")
        }
    }
}

const TRANSITION_TIME: Duration = Duration::from_secs(3);

#[derive(Component, Deref, DerefMut)]
struct TransitionTimer(Timer);
impl Default for TransitionTimer {
    fn default() -> Self {
        Self(Timer::new(TRANSITION_TIME, TimerMode::Once))
    }
}

fn phase2_transition(
    q_transition: Single<(Entity, &mut TransitionTimer, &mut Transform)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let (entity, mut timer, mut transform) = q_transition.into_inner();
    if timer.tick(time.delta()).just_finished() {
        commands.entity(entity).try_remove::<TransitionTimer>();
        commands.trigger(EnterPhase2 { entity });
    } else {
        let scale = (PHASE2_SIZE.x / SIZE.x - 1.0) * timer.fraction() + 1.0;
        transform.scale.x = scale;
        transform.scale.y = scale;
    }
}

fn enter_phase2_update_ruler(
    event: On<EnterPhase2>,
    mut q_ruler: Query<&mut enemy::health::EnemyHealth>,
    mut commands: Commands,
) {
    let Ok(mut enemy_health) = q_ruler.get_mut(event.entity) else {
        return;
    };
    enemy_health.value = PHASE2_HEALTH;
    commands.spawn((ChildOf(event.entity), phase2::RulerPhase2));
}
