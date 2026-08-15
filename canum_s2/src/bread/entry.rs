use super::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(BREAD_STATE.clone()), init_bread);
        app.add_systems(OnEnter(BREAD_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(spawn_title)));
            commands.spawn((SessionOnly, Observer::new(bread_start)));
            commands.spawn((SessionOnly, Observer::new(switch_stages)));
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
    commands.spawn((
        ChildOf(bread),
        (
            behaviors::BreadBehaviors,
            children![
                behaviors::BumpAround::default(),
                behaviors::SpeedyDash::default()
            ],
        ),
    ));

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

fn bread_start(_event: On<BreadStart>, mut commands: Commands) {
    commands.spawn((Music, Sound::new("Bread_Music")));
}

fn switch_stages(
    event: On<BreadNextStage>,
    q_bread: Query<&Children>,
    q_manager: Query<(), With<behaviors::BreadBehaviors>>,
    mut commands: Commands,
) {
    let Ok(children) = q_bread.get(event.entity) else {
        return;
    };
    let Some(manager) = children.iter().find(|child| q_manager.get(*child).is_ok()) else {
        return;
    };
    commands.entity(manager).despawn_children();
    match event.stage {
        2 => {
            commands.spawn((ChildOf(manager), behaviors::RandomShoot));
        }
        3 => {}
        _ => {}
    }
}
