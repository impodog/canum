mod behaviors;
mod entry;
mod obstacles;
mod wind;

use crate::prelude::*;

pub static WINDY_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Windy".to_owned()));

pub(super) struct WindyPlugin;

impl Plugin for WindyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            entry::EntryPlugin,
            wind::WindPlugin,
            obstacles::ObstaclesPlugin,
            behaviors::BehaviorsPlugin,
        ));
        app.add_systems(
            FixedUpdate,
            timer_tick.run_if(in_state(WINDY_STATE.clone())),
        );
        app.add_systems(OnEnter(WINDY_STATE.clone()), |mut commands: Commands| {
            commands.spawn(WindyMainEntity::default());
            commands.spawn((SessionOnly, Observer::new(spawn_timer_bar)));
            commands.spawn((SessionOnly, Observer::new(enter_stage2)));
            commands.spawn((SessionOnly, Observer::new(enter_stage3)));
            commands.spawn((SessionOnly, Observer::new(end_all_attack)));
            commands.insert_resource(WindyCurrentStage(1));
        });
    }
}

const TOTAL_TIME: f32 = 114.5;

#[derive(Component)]
#[require(SessionOnly, health::Friendly(false), player::victory::DefeatToWin)]
pub struct WindyMainEntity {
    pub timer: Timer,
    pub stage1: Timer,
    pub stage2: Timer,
    pub stage3: Timer,
    pub started: bool,
}

impl Default for WindyMainEntity {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(TOTAL_TIME, TimerMode::Once),
            stage1: Timer::from_seconds(49.44, TimerMode::Once),
            stage2: Timer::from_seconds(66.45, TimerMode::Once),
            stage3: Timer::from_seconds(108.80, TimerMode::Once),
            started: false,
        }
    }
}

fn spawn_timer_bar(
    _event: On<entry::WindyBegin>,
    mut commands: Commands,
    bottom_center: Single<Entity, With<canum_ui::BottomCenter>>,
    mut windy: Single<&mut WindyMainEntity>,
) {
    windy.started = true;
    commands.spawn((
        ChildOf(bottom_center.entity()),
        canum_ui::bar::health_bar(
            Color::srgb_u8(180, 180, 220),
            Color::srgb_u8(30, 30, 100),
            canum_ui::bar::HealthBar {
                total: TOTAL_TIME,
                current: TOTAL_TIME,
                ..default()
            },
        ),
    ));
}

#[derive(Resource)]
struct WindyCurrentStage(pub i32);

#[derive(Event, Default)]
struct EnterStage2;

#[derive(Event, Default)]
struct EnterStage3;

#[derive(Event, Default)]
struct StopAttacks;

/// This entity will be despawned when switching stages.
#[derive(Component, Default)]
struct StageDelete;

fn timer_tick(
    mut windy: Single<(
        Entity,
        &mut WindyMainEntity,
        &mut player::victory::DefeatToWin,
    )>,
    time: Res<Time>,
    mut commands: Commands,
    mut bar: Single<&mut canum_ui::bar::HealthBar>,
) {
    if !windy.1.started {
        return;
    }
    if windy.1.timer.tick(time.delta()).just_finished() {
        windy.2.defeated = true;
        commands
            .entity(windy.0)
            .remove::<player::victory::DefeatToWin>();
    }
    if windy.1.stage1.tick(time.delta()).just_finished() {
        commands.trigger(EnterStage2);
    }
    if windy.1.stage2.tick(time.delta()).just_finished() {
        commands.trigger(EnterStage3);
    }
    if windy.1.stage3.tick(time.delta()).just_finished() {
        commands.trigger(StopAttacks);
    }
    bar.current = windy.1.timer.remaining_secs();
}

fn enter_stage2(
    _event: On<EnterStage2>,
    q_delete: Query<Entity, With<StageDelete>>,
    mut commands: Commands,
    mut wind: ResMut<wind::WindVelocity>,
    mut stage: ResMut<WindyCurrentStage>,
) {
    for entity in q_delete.iter() {
        commands.entity(entity).try_despawn();
    }
    commands.spawn(canum_fx::transition::PureColor {
        destroy: None,
        color: Color::linear_rgb(0.7, 0.7, 0.75),
        duration: Duration::from_secs_f32(0.6),
        remove_self: true,
    });
    wind.target_velocity.x = -260.0;
    wind.friction = 0.2;
    stage.0 = 2;
}

fn enter_stage3(
    _event: On<EnterStage3>,
    q_delete: Query<Entity, With<StageDelete>>,
    mut commands: Commands,
    mut wind: ResMut<wind::WindVelocity>,
    mut q_player: Query<&mut wind::CanBeBlown, With<player::Player>>,
    mut stage: ResMut<WindyCurrentStage>,
) {
    for entity in q_delete.iter() {
        commands.entity(entity).try_despawn();
    }
    commands.spawn(canum_fx::transition::PureColor {
        destroy: None,
        color: Color::linear_rgb(0.7, 0.7, 0.8),
        duration: Duration::from_secs_f32(0.6),
        remove_self: true,
    });
    wind.target_velocity.x = 380.0 * rand_sign();
    wind.friction = 0.3;
    for mut can_be_blown in q_player.iter_mut() {
        can_be_blown.0 = 0.75;
    }
    stage.0 = 3;
}

fn end_all_attack(
    _event: On<StopAttacks>,
    q_delete: Query<Entity, With<StageDelete>>,
    q_manager: Query<Entity, With<behaviors::WindyBehaviors>>,
    mut commands: Commands,
    mut wind: ResMut<wind::WindVelocity>,
) {
    commands.spawn(canum_fx::transition::PureColor {
        destroy: None,
        color: Color::linear_rgb(0.7, 0.9, 0.7),
        duration: Duration::from_secs_f32(0.5),
        remove_self: true,
    });
    for entity in q_delete.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in q_manager.iter() {
        commands.entity(entity).try_despawn();
    }
    wind.target_velocity.x = 10.0;
    wind.friction = 1.0;
}
