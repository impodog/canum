use super::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(LASER_STATE.clone()), (spawn_laser, move_player));
    }
}

fn spawn_laser(mut commands: Commands) {
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
}

fn move_player(mut q_player: Query<&mut Transform, With<player::Player>>) {
    for mut transform in q_player.iter_mut() {
        transform.translation.x = -200.0;
    }
}
