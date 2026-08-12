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
            FixedPostUpdate,
            timer_tick.run_if(in_state(WINDY_STATE.clone())),
        );
        app.add_systems(OnEnter(WINDY_STATE.clone()), |mut commands: Commands| {
            commands.spawn(WindyMainEntity::default());
            commands.spawn((SessionOnly, Observer::new(spawn_timer_bar)));
        });
    }
}

const TOTAL_TIME: f32 = 114.5;

#[derive(Component)]
#[require(SessionOnly, health::Friendly(false), player::victory::DefeatToWin)]
pub struct WindyMainEntity {
    pub timer: Timer,
    pub started: bool,
}

impl Default for WindyMainEntity {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(TOTAL_TIME, TimerMode::Once),
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
            },
        ),
    ));
}

fn timer_tick(
    mut windy: Single<(Entity, &mut WindyMainEntity)>,
    time: Res<Time>,
    mut commands: Commands,
    mut bar: Single<&mut canum_ui::bar::HealthBar>,
) {
    if !windy.1.started {
        return;
    }
    if windy.1.timer.tick(time.delta()).just_finished() {
        commands
            .entity(windy.0)
            .remove::<player::victory::DefeatToWin>();
    }
    bar.current = windy.1.timer.remaining_secs();
}
