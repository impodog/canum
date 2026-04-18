use crate::prelude::*;

pub(super) struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(super::ANT_STATE.clone()), setup_ant);
        app.add_systems(
            FixedFirst,
            setup_timer.run_if(in_state(super::ANT_STATE.clone())),
        );
    }
}

#[derive(Resource, Debug, Deref, DerefMut)]
struct SetupTimer(Timer);
impl Default for SetupTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Once))
    }
}

#[derive(Event, Default)]
pub struct AntSetupTimerComplete;

fn setup_ant(
    mut commands: Commands,
    mut window_title: ResMut<canum_res::window::WindowTitle>,
    lang: Res<Lang>,
) {
    commands.insert_resource(SetupTimer::default());
    window_title.0 = lang.get("Ant_WindowTitle").to_owned();
    commands.spawn((
        canum_res::background::Background::new(vec2(800.0, 450.0)),
        Animation::new("Ant_Back", vec2(800.0, 450.0)).with_color(Color::default().with_alpha(0.8)),
    ));
}

fn setup_timer(mut commands: Commands, timer: Option<ResMut<SetupTimer>>, time: Res<Time>) {
    if let Some(mut timer) = timer {
        timer.tick(time.delta());
        if timer.just_finished() {
            commands.trigger(AntSetupTimerComplete);
            commands.remove_resource::<SetupTimer>();
        }
    }
}
