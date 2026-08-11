use super::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(WINDY_STATE.clone()), init_windy);
        app.add_systems(OnEnter(WINDY_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(spawn_title)));
            commands.spawn((SessionOnly, Observer::new(windy_begin)));
        });
    }
}

#[derive(Event, Default)]
struct SpawnTitle;
canum_fx::wait_then_trigger!(SpawnTitleTrigger, SpawnTitle, 1.0);

#[derive(Event, Default)]
pub struct WindyBegin;
canum_fx::wait_then_trigger!(WindyBeginTrigger, WindyBegin, 2.0);

fn init_windy(
    mut commands: Commands,
    mut window_title: ResMut<canum_res::window::WindowTitle>,
    lang: Res<Lang>,
) {
    commands
        .spawn(SpawnTitleTrigger)
        .observe(SpawnTitleTrigger::observer);
    commands
        .spawn(WindyBeginTrigger)
        .observe(WindyBeginTrigger::observer);
    window_title.0 = lang.get("Windy_WindowTitle").to_owned();
}

fn spawn_title(
    _event: On<SpawnTitle>,
    mut commands: Commands,
    lang: Res<Lang>,
    fonts: Res<canum_ui::Fonts>,
    bottom_left: Single<Entity, With<canum_ui::BottomLeft>>,
) {
    commands.spawn((
        ChildOf(bottom_left.entity()),
        canum_ui::text::popup_title(
            fonts.title.clone(),
            lang.get("Windy_BossTitle"),
            Duration::from_secs_f32(1.0),
        ),
    ));
}

fn windy_begin(_event: On<WindyBegin>, mut commands: Commands) {
    commands.spawn((Music, Sound::new("Windy_Bgm")));
}
