use super::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(LASER_STATE.clone()),
            (spawn_laser, move_player, spawn_laser_health_bar).chain(),
        );
    }
}

fn spawn_laser(
    mut commands: Commands,
    fonts: Res<canum_ui::Fonts>,
    lang: Res<Lang>,
    mut window_title: ResMut<canum_res::window::WindowTitle>,
    q_bottom_left: Query<Entity, With<canum_ui::BottomLeft>>,
) {
    commands.spawn((
        SessionOnly,
        canum_res::background::Background::new(CONFIG.display.screen_size),
        Animation::new("Laser_Back", CONFIG.display.screen_size)
            .with_color(Color::default().with_alpha(0.5)),
    ));

    let laser = commands.spawn((LaserBoss,)).id();
    commands.spawn((
        ChildOf(laser),
        enemy::health::EnemySensor,
        Collider::rectangle(60.0, 20.0),
    ));
    commands.spawn((
        ChildOf(laser),
        behaviors::LaserBehaviors,
        children![
            behaviors::ShootAndRotate::default(),
            behaviors::ScreenAttack::default()
        ],
    ));
    commands.spawn((SessionOnly, Music, Sound::new("Laser_Bgm")));

    let Ok(bottom_left) = q_bottom_left.single() else {
        return;
    };
    commands.spawn((
        ChildOf(bottom_left),
        canum_ui::text::popup_title(
            fonts.title.clone(),
            lang.get("Laser_BossTitle"),
            Duration::from_secs_f32(1.5),
        ),
    ));

    window_title.0 = lang.get("Laser_WindowTitle").to_owned();
}

fn move_player(mut q_player: Query<&mut Transform, With<player::Player>>) {
    for mut transform in q_player.iter_mut() {
        transform.translation.x = -200.0;
    }
}

fn spawn_laser_health_bar(
    save: Res<Save>,
    mut commands: Commands,
    q_bottom_center: Query<Entity, With<canum_ui::BottomCenter>>,
    mut title: ResMut<canum_res::window::WindowTitle>,
    lang: Res<Lang>,
    laser: Single<Entity, With<LaserBoss>>,
) {
    let Ok(bottom_center) = q_bottom_center.single() else {
        return;
    };
    title.0 = lang.get("Laser_WindowTitle").to_owned();
    if save.progress.selected_effects.contains("ShowHealth") {
        commands.spawn((
            ChildOf(bottom_center),
            canum_ui::bar::AssociatedBoss(laser.entity()),
            canum_ui::bar::health_bar(
                Color::Srgba(Srgba::hex("#ffb9db").unwrap()),
                Color::Srgba(Srgba::hex("#cc0000").unwrap()),
                canum_ui::bar::HealthBar {
                    total: 4000.0,
                    current: 4000.0,
                    ..default()
                },
            ),
        ));
    }
}
