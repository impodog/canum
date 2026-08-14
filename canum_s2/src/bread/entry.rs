use super::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(BREAD_STATE.clone()), init_bread);
        app.add_systems(OnEnter(BREAD_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(spawn_title)));
        });
    }
}

#[derive(Event, Default)]
pub struct BreadSpawnTitle;
canum_fx::wait_then_trigger!(BreadSpawnTitleTrigger, BreadSpawnTitle, 0.2);

#[derive(Event, Default)]
pub struct BreadStart;
canum_fx::wait_then_trigger!(BreadStartTrigger, BreadStart, 0.5);

fn init_bread(
    mut commands: Commands,
    mut q_player: Query<&mut Transform, With<player::Player>>,
    mut window_title: ResMut<canum_res::window::WindowTitle>,
    lang: Res<Lang>,
) {
    commands
        .spawn(BreadSpawnTitleTrigger)
        .observe(BreadSpawnTitleTrigger::observer);
    commands
        .spawn(BreadStartTrigger)
        .observe(BreadStartTrigger::observer);

    let bread = commands.spawn(BreadBoss::default()).id();
    commands.spawn((ChildOf(bread), (behaviors::BreadBehaviors, children![])));

    window_title.0 = lang.get("Bread_WindowTitle").to_owned();

    const MOVE_LENGTH: f32 = 150.0;
    for mut transform in q_player.iter_mut() {
        transform.translation +=
            vec3(MOVE_LENGTH, 0.0, 0.0).rotate_z(rand::random_range(0.0..std::f32::consts::TAU));
    }
}

fn spawn_title(
    _event: On<BreadSpawnTitle>,
    mut commands: Commands,
    fonts: Res<canum_ui::Fonts>,
    lang: Res<Lang>,
    bottom_left: Single<Entity, With<canum_ui::BottomLeft>>,
) {
    commands.spawn((
        ChildOf(bottom_left.entity()),
        canum_ui::text::popup_title(
            fonts.title.clone(),
            lang.get("Bread_BossTitle"),
            Duration::from_secs_f32(1.5),
        ),
    ));
}
