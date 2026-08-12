use super::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(WINDY_STATE.clone()), init_windy);
        app.add_systems(OnEnter(WINDY_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(spawn_title)));
            commands.spawn((SessionOnly, Observer::new(windy_begin)));
            commands.spawn((SessionOnly, Observer::new(spawn_wall_of_spikes)));
            commands.spawn((SessionOnly, Observer::new(move_wall_of_spikes)));
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

fn windy_begin(
    _event: On<WindyBegin>,
    mut commands: Commands,
    q_player: Query<Entity, With<player::Player>>,
) {
    commands.spawn((Music, Sound::new("Windy_Bgm")));
    for entity in q_player.iter() {
        commands.entity(entity).insert(wind::CanBeBlown::default());
    }
    commands.spawn((
        SessionOnly,
        behaviors::WindyBehaviors,
        children![
            behaviors::SpawnTumbleWeed,
            behaviors::PoleStorm::default(),
            behaviors::WaveOfSpikes,
            behaviors::TwoPoles::default(),
        ],
    ));
}

const SPIKE_LENGTH: f32 = 32.0;
const SPIKE_LENGTH_2: f32 = SPIKE_LENGTH * 0.5;

fn spawn_wall_of_spikes(_event: On<SpawnTitle>, mut commands: Commands) {
    // Spawn horizontal line of spikes
    let mut x = 0.0;
    while x < CONFIG.display.half_virtual_size.0 - SPIKE_LENGTH_2 {
        commands.spawn((
            obstacles::Spike(SPIKE_LENGTH),
            Transform::from_translation(vec3(
                x,
                CONFIG.display.half_virtual_size.1 + SPIKE_LENGTH_2,
                1.0,
            ))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
        ));
        commands.spawn((
            obstacles::Spike(SPIKE_LENGTH),
            Transform::from_translation(vec3(
                x,
                -CONFIG.display.half_virtual_size.1 - SPIKE_LENGTH_2,
                1.0,
            )),
        ));
        if x != 0.0 {
            commands.spawn((
                obstacles::Spike(SPIKE_LENGTH),
                Transform::from_translation(vec3(
                    -x,
                    CONFIG.display.half_virtual_size.1 + SPIKE_LENGTH_2,
                    1.0,
                ))
                .with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
            ));
            commands.spawn((
                obstacles::Spike(SPIKE_LENGTH),
                Transform::from_translation(vec3(
                    -x,
                    -CONFIG.display.half_virtual_size.1 - SPIKE_LENGTH_2,
                    1.0,
                )),
            ));
        }
        x += SPIKE_LENGTH;
    }

    let mut y = SPIKE_LENGTH_2;
    while y < CONFIG.display.half_virtual_size.1 {
        commands.spawn((
            obstacles::Spike(SPIKE_LENGTH),
            Transform::from_translation(vec3(
                CONFIG.display.half_virtual_size.0 + SPIKE_LENGTH_2,
                y,
                1.0,
            ))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        ));
        commands.spawn((
            obstacles::Spike(SPIKE_LENGTH),
            Transform::from_translation(vec3(
                -CONFIG.display.half_virtual_size.0 - SPIKE_LENGTH_2,
                y,
                1.0,
            ))
            .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
        ));
        commands.spawn((
            obstacles::Spike(SPIKE_LENGTH),
            Transform::from_translation(vec3(
                CONFIG.display.half_virtual_size.0 + SPIKE_LENGTH_2,
                -y,
                1.0,
            ))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        ));
        commands.spawn((
            obstacles::Spike(SPIKE_LENGTH),
            Transform::from_translation(vec3(
                -CONFIG.display.half_virtual_size.0 - SPIKE_LENGTH_2,
                -y,
                1.0,
            ))
            .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
        ));
        y += SPIKE_LENGTH;
    }
}

fn move_wall_of_spikes(
    _event: On<WindyBegin>,
    q_spike: Query<(Entity, &GlobalTransform), With<obstacles::Spike>>,
    mut commands: Commands,
) {
    for (entity, global_transform) in q_spike.iter() {
        let angle =
            global_transform.rotation().to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2;
        commands.spawn((
            ChildOf(entity),
            enemy::movements::Displacement {
                curve: |x| QuadraticInCurve.sample(x).unwrap(),
                displace: Vec2::from_angle(angle) * SPIKE_LENGTH,
                duration: Duration::from_secs_f32(0.2),
                notify: None,
            },
        ));
        commands.spawn(Sound::new("Windy_SpikeOut"));
    }
}
