use super::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PROJECTED_STATE.clone()), setup_projected);
    }
}

#[derive(Event, Default)]
pub struct ProjectedSpawn;
canum_fx::wait_then_trigger!(ProjectedSpawnTrigger, ProjectedSpawn, 0.2);

#[derive(Event, Default)]
pub struct ProjectedStart;
canum_fx::wait_then_trigger!(ProjectedStartTrigger, ProjectedStart, 2.0);

fn setup_projected(mut commands: Commands) {
    commands
        .spawn(ProjectedSpawnTrigger)
        .observe(ProjectedSpawnTrigger::observer);
    commands
        .spawn(ProjectedStartTrigger)
        .observe(ProjectedStartTrigger::observer);
    commands.spawn((SessionOnly, Observer::new(spawn_necessary)));
    commands.spawn(ProjectedMainEntity);

    commands.spawn((
        SessionOnly,
        canum_res::background::Background::new(CONFIG.display.screen_size),
        Animation::new("Projected_Back", CONFIG.display.screen_size),
    ));
}

const AMPLIFY_RATIO: f32 = 2.0;
const Y_DIFF: f32 = (200.0 - 55.0) * AMPLIFY_RATIO;

fn spawn_necessary(
    _event: On<ProjectedSpawn>,
    mut commands: Commands,
    mut title: ResMut<canum_res::window::WindowTitle>,
    primary_player: Res<player::PrimaryPlayer>,
    lang: Res<Lang>,
    fonts: Res<canum_ui::Fonts>,
    bottom_left: Single<Entity, With<canum_ui::BottomLeft>>,
) {
    commands.spawn((
        ChildOf(primary_player.0),
        flashlight::FlashlightOverlay {
            amplify_ratio: AMPLIFY_RATIO,
        },
        Transform::from_translation(vec3(0.0, Y_DIFF, 26.0))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
    ));
    title.0 = lang.get("Projected_WindowTitle").to_owned();
    commands.spawn((
        ChildOf(bottom_left.entity()),
        canum_ui::text::popup_title(
            fonts.title.clone(),
            lang.get("Projected_BossTitle"),
            Duration::from_secs_f32(2.0),
        ),
    ));
    commands.spawn((Music, Sound::new("Projected_Bgm")));
}
