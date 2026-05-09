use super::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(RUNWAY_STATE.clone()), init_timer);
        app.add_systems(
            FixedFirst,
            timer_tick.run_if(in_state(RUNWAY_STATE.clone())),
        );
    }
}

#[derive(Resource, Deref, DerefMut)]
struct EntryTimer(Timer);
impl Default for EntryTimer {
    fn default() -> Self {
        EntryTimer(Timer::from_seconds(0.2, TimerMode::Once))
    }
}

#[derive(Event, Default)]
pub struct StartRunwaySetup;

#[derive(Event, Default)]
pub struct RunwayOfficialStart;

fn init_timer(mut commands: Commands) {
    commands.insert_resource(EntryTimer::default());
    commands.spawn((SessionOnly, Observer::new(setup_runway)));
}
fn timer_tick(mut commands: Commands, mut timer: ResMut<EntryTimer>, time: Res<Time>) {
    if !timer.is_finished() {
        timer.tick(time.delta());
        if timer.just_finished() {
            commands.trigger(StartRunwaySetup);
        }
    }
}

fn setup_runway(
    _event: On<StartRunwaySetup>,
    mut q_player: Query<&mut Transform, With<player::Player>>,
    mut commands: Commands,
    mut window_title: ResMut<canum_res::window::WindowTitle>,
    lang: Res<Lang>,
) {
    for mut transform in q_player.iter_mut() {
        transform.translation.x += CONFIG.display.screen_size.x * 0.25;
    }
    commands.spawn((
        SessionOnly,
        Collider::rectangle(3.0, CONFIG.display.screen_size.y),
        RigidBody::Static,
    ));
    let lion_entity = commands
        .spawn((
            RunwayLion,
            Animation::new("Runway_Lion_Running", Vec2::new(64.0, 64.0)),
            Transform::from_translation(Vec3::new(
                -CONFIG.display.screen_size.x * 0.25,
                CONFIG.display.screen_size.y * 0.5,
                14.37,
            )),
            projectile::NoCollideBoundary,
        ))
        .observe(lion_reached_destination)
        .id();
    commands.spawn((
        ChildOf(lion_entity),
        enemy::movements::Displacement {
            curve: |x| QuadraticInOutCurve.sample(x).unwrap(),
            displace: vec2(0.0, CONFIG.display.screen_size.y * -0.5),
            duration: Duration::from_secs_f32(2.0),
            notify: Some(lion_entity),
        },
    ));
    window_title.0 = lang.get("Runway_WindowTitle").to_owned();
}

fn lion_reached_destination(
    event: On<enemy::movements::DisplacementComplete>,
    mut q_lion: Query<&mut Animation>,
    mut commands: Commands,
    q_bottom_left: Query<Entity, With<canum_ui::BottomLeft>>,
    fonts: Res<canum_ui::Fonts>,
    lang: Res<Lang>,
) {
    info!("Completed!");
    let Ok(mut animation) = q_lion.get_mut(event.entity) else {
        return;
    };
    animation.replace("Runway_Lion_Posing", false, None);
    commands
        .entity(event.entity)
        .remove::<projectile::NoCollideBoundary>();

    commands
        .spawn(canum_fx::util::WaitInterval::new(Duration::from_secs_f32(
            3.0,
        )))
        .observe(game_start);

    const BUMPER_HEIGHT: f32 = 10.0;
    commands.spawn((
        Transform::from_translation(Vec3::new(
            0.0,
            CONFIG.display.half_virtual_size.1 - BUMPER_HEIGHT * 0.5,
            0.0,
        )),
        Sensor,
        health::Friendly(false),
        Collider::rectangle(CONFIG.display.screen_size.x, BUMPER_HEIGHT),
        health::ContactDamage {
            value: 1000,
            order: consts::order::HEALTH_INVINC - 1,
            projectile: false,
        },
        obstacle::BumpAway {
            direction: Dir2::from_xy_unchecked(0.0, -1.0),
            strength: 30.0,
        },
    ));

    let Ok(bottom_left) = q_bottom_left.single() else {
        return;
    };
    commands.spawn((
        ChildOf(bottom_left),
        canum_ui::text::popup_title(
            fonts.title.clone(),
            lang.get("Runway_BossTitle"),
            Duration::from_secs_f32(2.5),
        ),
    ));
}

fn game_start(
    _event: On<canum_fx::util::WaitComplete>,
    mut commands: Commands,
    mut q_lion: Query<(Entity, &mut Animation), With<RunwayLion>>,
    mut speed: ResMut<running::RollingSpeed>,
) {
    const ROLLING_SPEED: f32 = 130.0;
    commands.trigger(RunwayOfficialStart);
    commands.spawn((Music, Sound::new("Runway_Bgm")));
    commands.spawn(RunwayObstacles);
    let Ok((entity, mut animation)) = q_lion.single_mut() else {
        return;
    };
    animation.replace("Runway_Lion_Running", false, None);
    commands.spawn((
        ChildOf(entity),
        movements::PartialVelocity {
            velocity: Vec2::new(0.0, -ROLLING_SPEED),
            linked: None,
        },
    ));
    speed.0 = ROLLING_SPEED;
}
