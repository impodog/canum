use super::*;
use enemy::behavior::*;
use rand::RngExt;

pub(super) struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<SpawnTumbleWeed>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(spawn_tumble_weed);
            });

        app.world_mut()
            .register_component_hooks::<WaveOfSpikes>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(wave_of_spikes_start);
            });

        app.world_mut()
            .register_component_hooks::<PoleStorm>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(pole_storm);
            });
        app.add_systems(
            FixedUpdate,
            pole_storm_work.run_if(in_state(WINDY_STATE.clone())),
        );

        app.world_mut()
            .register_component_hooks::<TwoPoles>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(two_poles_start);
            });
        app.add_systems(
            FixedUpdate,
            two_poles_work.run_if(in_state(WINDY_STATE.clone())),
        );

        app.add_systems(OnEnter(WINDY_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(wave_of_spikes)));
            commands.spawn((SessionOnly, Observer::new(two_poles_spawn)));
        });
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::default())]
pub struct WindyBehaviors;

#[derive(Component, Default)]
#[require(Behavior::new("Windy_SpawnTumbleWeed", 1.5, ["SpawnTumbleWeed"]))]
pub struct SpawnTumbleWeed;

fn spawn_tumble_weed(
    event: On<BehaveStart>,
    mut commands: Commands,
    wind: Option<Res<wind::WindVelocity>>,
    stage: Res<WindyCurrentStage>,
) {
    const RADIUS: f32 = obstacles::TumbleWeed::RADIUS;
    if let Some(wind) = wind {
        let sgn = wind.target_velocity.x.signum();
        let vertical_speed = rand_normal(0.0, 20.0);
        let tumble_weed = commands
            .spawn((
                obstacles::TumbleWeed,
                wind::CanBeBlown(1.3),
                Transform::from_translation(vec3(
                    (CONFIG.display.half_virtual_size.0 + RADIUS) * -sgn,
                    rand::random_range(
                        -CONFIG.display.half_virtual_size.1 + RADIUS
                            ..CONFIG.display.half_virtual_size.1 - RADIUS,
                    ),
                    2.1,
                )),
            ))
            .id();
        commands.spawn((
            ChildOf(tumble_weed),
            movements::PartialVelocity::unlinked().with_velocity(vec2(0.0, vertical_speed)),
        ));
    }
    let rest_time = if stage.0 == 1 {
        rand_normal(1.0, 0.2).clamp(0.5, 1.5)
    } else {
        rand_normal(0.8, 0.3).clamp(0.4, 1.5)
    };
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(0.1),
        occupies: occupies![("SpawnTumbleWeed", rest_time)],
    });
}

#[derive(Component, Default)]
#[require(Behavior::new("Windy_WaveOfSpikes", 0.8, ["Fullscreen", "WaveOfSpikes"]))]
pub struct WaveOfSpikes;

/// Marks the spikes spawn by this.
#[derive(Component)]
struct WaveOfSpikesMarker;

#[derive(Event, Default)]
struct WaveOfSpikesWarningComplete;

canum_fx::wait_then_trigger!(WaveOfSpikesWarningTrigger, WaveOfSpikesWarningComplete, 1.0);

fn wave_of_spikes_start(
    event: On<BehaveStart>,
    mut commands: Commands,
    wind: Option<Res<wind::WindVelocity>>,
) {
    let trigger = commands
        .spawn((
            WaveOfSpikesWarningTrigger,
            Transform::default(),
            Visibility::default(),
        ))
        .observe(WaveOfSpikesWarningTrigger::observer)
        .id();
    if let Some(wind) = wind {
        let sgn = wind.target_velocity.x.signum();
        commands.spawn((
            ChildOf(trigger),
            Animation::new("Windy_Warning", vec2(64.0, 64.0)),
            Transform::from_translation(vec3(
                (CONFIG.display.half_virtual_size.0 - 32.0) * sgn,
                0.0,
                15.0,
            )),
        ));
        commands.spawn(Sound::new("Windy_Warning"));
    }
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(0.1),
        occupies: occupies![
            ("WaveOfSpikes", rand_normal(9.5, 1.0)),
            ("Fullscreen", rand_normal(6.0, 0.5).max(6.0))
        ],
    });
}

fn wave_of_spikes(
    _event: On<WaveOfSpikesWarningComplete>,
    wind: Option<Res<wind::WindVelocity>>,
    mut commands: Commands,
    stage: Res<WindyCurrentStage>,
) {
    const SPIKE_LENGTH: f32 = 16.0;
    const HALF_SPIKE_LENGTH: f32 = SPIKE_LENGTH * 0.5;
    if let Some(wind) = wind {
        let sgn = wind.target_velocity.x.signum();
        let x = sgn * (CONFIG.display.half_virtual_size.0 + HALF_SPIKE_LENGTH);
        let mut y = HALF_SPIKE_LENGTH;
        let angle = sgn * std::f32::consts::FRAC_PI_2;
        let velocity = vec2(-sgn * if stage.0 == 3 { 550.0 } else { 450.0 }, 0.0);
        while y < CONFIG.display.half_virtual_size.1 {
            let entity1 = commands
                .spawn((
                    obstacles::Spike(SPIKE_LENGTH),
                    wind::CanBeBlown(2.8),
                    WaveOfSpikesMarker,
                    Transform::from_translation(vec3(x, y, 1.2))
                        .with_rotation(Quat::from_rotation_z(angle)),
                ))
                .id();
            let entity2 = commands
                .spawn((
                    obstacles::Spike(SPIKE_LENGTH),
                    wind::CanBeBlown(2.8),
                    WaveOfSpikesMarker,
                    Transform::from_translation(vec3(x, -y, 1.2))
                        .with_rotation(Quat::from_rotation_z(angle)),
                ))
                .id();
            for entity in [entity1, entity2] {
                commands.entity(entity).insert(StageDelete);
                commands.spawn((
                    ChildOf(entity),
                    movements::PartialVelocity::unlinked().with_velocity(velocity),
                ));
            }
            y += SPIKE_LENGTH;
        }
    }
}

#[derive(Component)]
#[require(Behavior::new("Windy_PoleStorm", 0.6, ["PoleStorm", "Fullscreen"]))]
pub struct PoleStorm {
    pub remaining: usize,
    pub timer: Timer,
    pub position: f32,
    pub speed: f32,
}
impl Default for PoleStorm {
    fn default() -> Self {
        Self {
            remaining: 0,
            timer: Timer::from_seconds(POLE_STORM_INTERVAL, TimerMode::Repeating),
            position: 0.0,
            speed: 0.0,
        }
    }
}
const POLE_STORM_TIMES: usize = 20;
const POLE_STORM_INTERVAL: f32 = 0.28;

fn pole_storm(
    event: On<BehaveStart>,
    mut commands: Commands,
    mut q_pole_storm: Query<&mut PoleStorm>,
) {
    let Ok(mut pole_storm) = q_pole_storm.get_mut(event.entity) else {
        return;
    };
    pole_storm.remaining += POLE_STORM_TIMES;
    pole_storm.position = rand_normal(0.0, 50.0);
    pole_storm.speed = rand_normal(50.0, 5.0) * rand_sign();
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(0.1),
        occupies: occupies![
            ("PoleStorm", rand_normal(10.0, 1.2).min(12.0)),
            (
                "SpawnTumbleWeed",
                POLE_STORM_TIMES as f32 * POLE_STORM_INTERVAL
            ),
            ("Fullscreen", rand_normal(8.0, 0.5).clamp(7.5, 9.0))
        ],
    });
}

fn pole_storm_work(
    mut q_pole_storm: Query<&mut PoleStorm>,
    time: Res<Time>,
    mut commands: Commands,
    wind: Option<Res<wind::WindVelocity>>,
) {
    let limit_size = CONFIG.display.half_virtual_size.1 - 64.0;
    let Some(wind) = wind else {
        return;
    };
    let sgn = wind.target_velocity.x.signum();
    for mut pole_storm in q_pole_storm.iter_mut() {
        if pole_storm.remaining == 0 {
            continue;
        }
        if pole_storm.timer.tick(time.delta()).just_finished() {
            let x = -sgn * (CONFIG.display.half_virtual_size.0 + obstacles::Pole::HALF_SIZE.x);
            commands.spawn((
                obstacles::Pole { face_down: true },
                wind::CanBeBlown(2.2),
                Transform::from_translation(vec3(x, pole_storm.position - 65.0, 1.3)),
            ));
            commands.spawn((
                obstacles::Pole { face_down: false },
                wind::CanBeBlown(2.2),
                Transform::from_translation(vec3(x, pole_storm.position + 65.0, 1.3)),
            ));
            pole_storm.remaining = pole_storm.remaining.saturating_sub(1);
        }
        pole_storm.position += pole_storm.speed * time.delta_secs();
        if pole_storm.position > limit_size {
            pole_storm.position = limit_size * 2.0 - pole_storm.position;
            pole_storm.speed = -pole_storm.speed;
        }
        if pole_storm.position < -limit_size {
            pole_storm.position = -limit_size * 2.0 - pole_storm.position;
            pole_storm.speed = -pole_storm.speed;
        }
    }
}

#[derive(Component, Default)]
#[require(Behavior::new("Windy_TwoPoles", 0.7, ["Fullscreen", "TwoPoles"]))]
pub struct TwoPoles {
    left_up: bool,
}
impl TwoPoles {
    const LENGTH: f32 = 300.0;
    const HALF_LENGTH: f32 = Self::LENGTH * 0.5;
    const ACCELERATION: f32 = 70.0;
    const STAGE3_ACCELERATION: f32 = 110.0;
    const INITIAL_SPEED: f32 = 240.0;
    const STAGE3_INITIAL_SPEED: f32 = 300.0;
}

/// Stores the initial speed sign of the pole.
#[derive(Component)]
struct TwoPolesMarker(f32);

#[derive(Event)]
struct TwoPolesWarningComplete(Entity);
canum_fx::wait_then_trigger!(
    TwoPolesWarningCompleteTrigger,
    TwoPolesWarningComplete,
    Entity,
    1.0
);

fn two_poles_start(
    event: On<BehaveStart>,
    mut commands: Commands,
    mut q_two_poles: Query<&mut TwoPoles>,
) {
    let trigger = commands
        .spawn((
            TwoPolesWarningCompleteTrigger(event.entity),
            Transform::default(),
            Visibility::default(),
        ))
        .observe(TwoPolesWarningCompleteTrigger::observer)
        .id();

    let Ok(mut two_poles) = q_two_poles.get_mut(event.entity) else {
        return;
    };
    let left_up = rand::rng().random_ratio(1, 2);
    two_poles.left_up = left_up;
    let positions = [
        (
            CONFIG.display.half_virtual_size.1 - TwoPoles::HALF_LENGTH,
            -CONFIG.display.half_virtual_size.0 + 32.0,
        ),
        (
            TwoPoles::HALF_LENGTH - CONFIG.display.half_virtual_size.1,
            CONFIG.display.half_virtual_size.0 - 32.0,
        ),
    ];
    let sgn = if left_up { 1.0 } else { -1.0 };
    for (y, x) in positions {
        commands.spawn((
            ChildOf(trigger),
            Animation::new("Windy_Warning", vec2(64.0, 64.0)),
            Transform::from_translation(vec3(x, y * sgn, 15.0)),
        ));
    }
    commands.spawn(Sound::new("Windy_Warning"));

    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(0.5),
        occupies: occupies![
            ("TwoPoles", rand_normal(10.5, 1.0).clamp(9.0, 12.5)),
            ("Fullscreen", rand_normal(9.0, 0.7).max(8.0))
        ],
    });
}

fn two_poles_spawn(
    event: On<TwoPolesWarningComplete>,
    q_two_poles: Query<&TwoPoles>,
    mut commands: Commands,
    stage: Res<WindyCurrentStage>,
) {
    let Ok(two_poles) = q_two_poles.get(event.0) else {
        return;
    };
    let sgn = if two_poles.left_up { 1.0 } else { -1.0 };
    let positions = [
        (
            CONFIG.display.half_virtual_size.1 - TwoPoles::LENGTH,
            -CONFIG.display.half_virtual_size.0 - obstacles::Pole::HALF_SIZE.x,
            false,
        ),
        (
            TwoPoles::LENGTH - CONFIG.display.half_virtual_size.1,
            CONFIG.display.half_virtual_size.0 + obstacles::Pole::HALF_SIZE.x,
            true,
        ),
    ];
    let initial_speed = if stage.0 == 3 {
        TwoPoles::STAGE3_INITIAL_SPEED
    } else {
        TwoPoles::INITIAL_SPEED
    };
    for (y, x, down) in positions {
        commands.spawn((
            obstacles::Pole {
                face_down: down ^ !two_poles.left_up,
            },
            TwoPolesMarker(-x.signum()),
            Transform::from_translation(vec3(x, y * sgn, 1.3)),
            LinearVelocity(vec2(-x.signum() * initial_speed, 0.0)),
        ));
    }
}

fn two_poles_work(
    mut q_two_poles: Query<(&mut LinearVelocity, &TwoPolesMarker)>,
    time: Res<Time>,
    stage: Res<WindyCurrentStage>,
) {
    let acceleration = if stage.0 == 3 {
        TwoPoles::STAGE3_ACCELERATION
    } else {
        TwoPoles::ACCELERATION
    };
    for (mut linear_velocity, marker) in q_two_poles.iter_mut() {
        linear_velocity.x -= marker.0 * acceleration * time.delta_secs();
    }
}
