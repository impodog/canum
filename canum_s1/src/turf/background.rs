use crate::prelude::*;

pub(super) struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SetupTimer>();
        app.add_systems(OnEnter(super::TURF_STATE.clone()), setup_turf);
        app.add_systems(
            FixedPreUpdate,
            setup_timer.run_if(in_state(super::TURF_STATE.clone())),
        );
        app.add_observer(spawn_turf_title);
    }
}

/// Wait for a moment before setting up. This ensure all relevant entities are spawned.
#[derive(Resource, Debug, Deref, DerefMut)]
struct SetupTimer(Timer);
impl Default for SetupTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Once))
    }
}

#[derive(Event, Default)]
pub struct TurfSetupTimerComplete;

fn setup_turf(
    mut commands: Commands,
    mut window_title: ResMut<canum_res::window::WindowTitle>,
    lang: Res<Lang>,
) {
    commands.spawn((
        SessionOnly,
        canum_res::background::Background::new(CONFIG.display.screen_size),
        Animation::new("Turf_Grassland", CONFIG.display.screen_size)
            .with_color(Color::default().with_alpha(0.8)),
    ));
    commands.spawn((Music, Sound::new("Turf_Bgm")));
    commands.spawn((SessionOnly, Observer::new(change_turf_background_on_defeat)));
    window_title.0 = lang.get("Turf_WindowTitle").to_owned();
    commands.insert_resource(SetupTimer::default());
}

fn change_turf_background_on_defeat(
    _event: On<player::victory::PlayerWin>,
    mut q_background: Query<&mut Animation, With<canum_res::background::Background>>,
) {
    let Ok(mut animation) = q_background.single_mut() else {
        return;
    };
    animation.replace("Turf_GrasslandDefeat", false, None);
}

fn setup_timer(timer: Option<ResMut<SetupTimer>>, time: Res<Time>, mut commands: Commands) {
    if let Some(mut timer) = timer {
        timer.tick(time.delta());
        if timer.just_finished() {
            commands.trigger(TurfSetupTimerComplete);
            commands.remove_resource::<SetupTimer>();
        }
    }
}

fn spawn_turf_title(
    _event: On<TurfSetupTimerComplete>,
    mut commands: Commands,
    q_bottom_left: Query<Entity, With<canum_ui::BottomLeft>>,
    fonts: Res<canum_ui::Fonts>,
    lang: Res<Lang>,
) {
    let Ok(bottom_left) = q_bottom_left.single() else {
        return;
    };
    commands.spawn((
        ChildOf(bottom_left),
        canum_ui::text::popup_title(
            fonts.title.clone(),
            lang.get("Turf_BossTitle"),
            Duration::from_secs(2),
        ),
    ));
}
