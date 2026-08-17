mod background;
mod behaviors;
mod defeat;

use crate::prelude::*;

static TURF_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Turf".to_owned()));

pub(super) struct TurfPlugin;

impl Plugin for TurfPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            background::BackgroundPlugin,
            behaviors::BehaviorsPlugin,
            defeat::DefeatPlugin,
        ));
        app.add_systems(
            FixedPreUpdate,
            (turf_count_down, update_health_bar)
                .chain()
                .run_if(in_state(TURF_STATE.clone())),
        );
        app.add_observer(spawn_turf);
    }
}

/// This is a timer count-down boss, therefore it has no health component.
#[derive(Component, Debug)]
#[require(health::Friendly(false), player::victory::DefeatToWin::default())]
pub struct TurfBoss {
    pub timer: Timer,
}
impl TurfBoss {
    pub const TIME: Duration = Duration::from_secs(90);
}
impl Default for TurfBoss {
    fn default() -> Self {
        Self {
            timer: Timer::new(TurfBoss::TIME, TimerMode::Once),
        }
    }
}

fn turf_count_down(
    mut commands: Commands,
    mut q_turf: Query<(Entity, &mut TurfBoss, &mut player::victory::DefeatToWin)>,
    time: Res<Time<Real>>,
) {
    for (entity, mut turf, mut defeat_to_win) in q_turf.iter_mut() {
        turf.timer.tick(time.delta());
        if turf.timer.is_finished() {
            defeat_to_win.defeated = true;
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
struct TurfTimerBar;

fn spawn_turf(
    _event: On<background::TurfSetupTimerComplete>,
    mut commands: Commands,
    q_bottom_center: Query<Entity, With<canum_ui::BottomCenter>>,
) {
    let Ok(bottom_center) = q_bottom_center.single() else {
        return;
    };
    let turf = commands.spawn((TurfBoss::default(),)).id();
    commands.spawn((
        ChildOf(bottom_center),
        TurfTimerBar,
        canum_ui::bar::health_bar(
            Color::linear_rgb(0.2, 1.0, 0.2),
            Color::linear_rgb(1.0, 0.1, 0.0),
            canum_ui::bar::HealthBar {
                total: TurfBoss::TIME.as_secs_f32(),
                current: TurfBoss::TIME.as_secs_f32(),
                ..default()
            },
        ),
    ));
    commands.spawn((ChildOf(turf), behaviors::TurfBehaviors));
}

fn update_health_bar(
    mut q_health_bar: Query<&mut canum_ui::bar::HealthBar, With<TurfTimerBar>>,
    q_turf: Query<&TurfBoss>,
) {
    let Ok(turf) = q_turf.single() else {
        return;
    };
    let Ok(mut health_bar) = q_health_bar.single_mut() else {
        return;
    };
    health_bar.current = turf.timer.remaining_secs();
}
