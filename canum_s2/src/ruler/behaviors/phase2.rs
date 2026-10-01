use super::*;

pub(super) struct Phase2Plugin;

impl Plugin for Phase2Plugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<RulerPhase2>()
            .on_add(ruler_phase2_hook);
        app.world_mut()
            .register_component_hooks::<HorizLaserAcc>()
            .on_add(horiz_laser_acc_hook);
        app.world_mut()
            .register_component_hooks::<ManyBallsFall>()
            .on_add(many_balls_fall_hook);
        app.world_mut()
            .register_component_hooks::<Platforms>()
            .on_add(platforms_hook);
        app.world_mut()
            .register_component_hooks::<BigLasers>()
            .on_add(big_lasers_hook);
        app.add_systems(
            FixedUpdate,
            (
                horiz_laser_accelerate,
                platforms_spawn,
                platforms_bar_accelerate,
            )
                .in_set(RulerSet),
        );
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct RulerPhase2;

fn ruler_phase2_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let mut commands = world.commands();
    commands.spawn((ChildOf(entity), HorizLaserAcc));
    commands.spawn((ChildOf(entity), ManyBallsFall));
    commands.spawn((ChildOf(entity), Platforms::default()));
    commands.spawn((ChildOf(entity), BigLasers));
    canum_fx::session_observers!(
        commands,
        horiz_laser_observe,
        horiz_laser_warning,
        many_balls_fall_action,
        platforms_despawn_child,
        big_lasers_spawn,
        big_lasers_despawn,
    );
}

#[derive(Component, Default)]
#[require(Behavior::new("Ruler_HorizLaserAcc", 1.0, ["Main", "HorizLaserAcc"]))]
struct HorizLaserAcc;

fn horiz_laser_acc_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let mut commands = world.commands();
    commands.entity(entity).observe(horiz_laser_acc_start);
}

#[derive(Event)]
struct HorizLaserAccSpawnLaser(f32);
impl HorizLaserAccSpawnLaser {
    fn new(value: f32) -> Self {
        Self(value)
    }
}

#[derive(Event)]
struct HorizLaserAccWarning(f32);
impl HorizLaserAccWarning {
    fn new(value: f32) -> Self {
        Self(value)
    }
}

#[derive(Event, Default)]
struct OneSecond;
canum_fx::wait_then_trigger!(OneSecondTrigger, OneSecond, 1.0);

fn horiz_laser_acc_start(event: On<BehaveStart>, mut commands: Commands) {
    let start_sign = rand_sign();
    canum_fx::wait_then_trigger!(use commands, HorizLaserAccWarning, f32, start_sign, 0.1);
    canum_fx::wait_then_trigger!(use commands, HorizLaserAccSpawnLaser, f32, start_sign, 1.1);
    canum_fx::wait_then_trigger!(use commands, HorizLaserAccWarning, f32, -start_sign, 1.92);
    canum_fx::wait_then_trigger!(use commands, HorizLaserAccSpawnLaser, f32, -start_sign, 2.92);
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(rand_normal(4.75, 0.5)),
        occupies: occupies![("HorizLaserAcc", rand_normal(6.0, 0.32))],
    });
    if rand_bool(0.4) {
        commands.trigger(BehaveQueue::new(event.entity, "Ruler_BigLaser"));
    }
}

fn horiz_laser_warning(event: On<HorizLaserAccWarning>, mut commands: Commands) {
    let trigger = commands
        .spawn((OneSecondTrigger, Transform::default(), Visibility::Visible))
        .observe(OneSecondTrigger::observer)
        .id();
    commands.spawn((
        ChildOf(trigger),
        Animation::new("Windy_Warning", vec2(64.0, 64.0)),
        Transform::from_translation(vec3(
            (CONFIG.display.half_virtual_size.0 - 32.0) * event.0,
            0.0,
            15.0,
        )),
    ));
    commands.spawn(Sound::new("Windy_Warning"));
}

#[derive(Component, Default)]
struct HorizLaser {
    acc: f32,
}

fn horiz_laser_observe(event: On<HorizLaserAccSpawnLaser>, mut commands: Commands) {
    let position = vec3(
        event.0 * (CONFIG.display.half_virtual_size.0 + 10.0),
        CONFIG.display.half_virtual_size.1 - 1.0,
        5.0,
    );
    let laser = commands
        .spawn((
            HorizLaser {
                acc: 310.0 * event.0,
            },
            Transform::from_translation(position),
            RigidBody::Kinematic,
            LinearVelocity(vec2(-event.0 * 550.0, 0.0)),
            super::laser::RulerLaser {
                direction: vec2(0.0, -1.0),
                double: false,
            },
        ))
        .id();
    commands.spawn((ChildOf(laser), Sound::new("Ruler_Laser")));
}
fn horiz_laser_accelerate(mut q_laser: Query<(&mut LinearVelocity, &HorizLaser)>, time: Res<Time>) {
    for (mut linear_velocity, horiz_laser) in q_laser.iter_mut() {
        linear_velocity.x += time.delta_secs() * horiz_laser.acc;
    }
}

#[derive(Component, Debug)]
#[require(Behavior::new("Ruler_ManyBallsFall", 0.8, ["Main", "ManyBallsFall"]))]
struct ManyBallsFall;

#[derive(Event)]
struct ManyBallsFallAction(Entity);
impl ManyBallsFallAction {
    fn new(entity: Entity) -> Self {
        Self(entity)
    }
}

fn many_balls_fall_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world
        .commands()
        .entity(entity)
        .observe(many_balls_fall_start);
}

fn many_balls_fall_start(event: On<BehaveStart>, mut commands: Commands) {
    let trigger = commands
        .spawn((OneSecondTrigger, Transform::default(), Visibility::Visible))
        .observe(OneSecondTrigger::observer)
        .id();
    commands.spawn((
        ChildOf(trigger),
        Animation::new("Windy_Warning", vec2(64.0, 64.0)),
        Transform::from_translation(vec3(0.0, CONFIG.display.half_virtual_size.1 - 32.0, 15.0)),
    ));
    commands.spawn(Sound::new("Windy_Warning"));

    canum_fx::wait_then_trigger!(use commands, ManyBallsFallAction, Entity, event.entity, 0.8);

    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(3.0),
        occupies: occupies![("ManyBallsFall", rand_normal(6.0, 0.7))],
    });
    if rand_bool(0.2) {
        commands.trigger(BehaveQueue::new(event.entity, "Ruler_BigLaser"));
    }
}

fn many_balls_fall_action(
    event: On<ManyBallsFallAction>,
    mut commands: Commands,
    ruler: Single<Entity, With<RulerBoss>>,
) {
    let sgn = rand_sign();
    let acceleration = rand_normal(300.0, 10.0);
    let speed = rand_range(310.0..370.0);
    let gap = rand_normal(125.0, 5.0);
    let distance = CONFIG.display.screen_size.y - PHASE2_SIZE.y;
    let time = (distance * 2.0 / acceleration).sqrt();
    let initial_displace = -speed * time
        + rand_normal(
            CONFIG.display.half_virtual_size.0 - phase1::BALL_RADIUS,
            20.0,
        );
    let count = (CONFIG.display.screen_size.x / gap).ceil() as usize;
    let mut displace = vec3(
        initial_displace * -sgn,
        CONFIG.display.half_virtual_size.1 + phase1::BALL_RADIUS,
        3.0,
    );
    for _ in 0..count {
        commands.spawn((
            phase1::Ball { bounce_times: 1 },
            Transform::from_translation(displace),
            health::AlwaysCollide::new([ruler.entity()]),
            projectile::NoCollideBoundary,
            LinearVelocity(vec2(-sgn * speed, 0.0)),
            ConstantLinearAcceleration(vec2(0.0, -acceleration)),
            canum_fx::util::DespawnCheck::new(event.0),
        ));
        displace.x += sgn * gap;
    }
}

#[derive(Component, Default)]
#[require(Behavior::new("Ruler_Platforms", 1.0, ["Main", "Platforms"]))]
struct Platforms {
    length: f32,
    order: Vec<f32>,
    child: Vec<Entity>,
    sign: f32,
    interval: Timer,
}

fn platforms_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world.commands().entity(entity).observe(platforms_start);
}

const SPIKE_LENGTH: f32 = 32.0;

fn platforms_start(
    event: On<BehaveStart>,
    mut commands: Commands,
    mut q_platforms: Query<&mut Platforms>,
) {
    use rand::seq::SliceRandom;

    let Ok(mut platforms) = q_platforms.get_mut(event.entity) else {
        return;
    };

    for child in platforms.child.drain(..) {
        commands.entity(child).try_despawn();
    }

    let count = rand_range(3..=4);
    platforms.length = (CONFIG.display.screen_size.y - PHASE2_SIZE.y) / count as f32;
    platforms.order.clear();
    platforms.sign = rand_sign();
    platforms.interval = Timer::from_seconds(rand_normal(0.5, 0.02), TimerMode::Repeating);
    let initial_elapsed = platforms.interval.duration() / 2;
    platforms.interval.set_elapsed(initial_elapsed);
    let mut current_y = CONFIG.display.half_virtual_size.1 - platforms.length * 0.5;
    for _ in 0..count {
        platforms.order.push(current_y);
        platforms.order.push(current_y);
        current_y -= platforms.length;
    }
    platforms.order.shuffle(&mut rand::rng());

    let spike_count =
        ((CONFIG.display.screen_size.y - PHASE2_SIZE.y) / SPIKE_LENGTH).ceil() as usize;
    current_y = CONFIG.display.half_virtual_size.1 - SPIKE_LENGTH * 0.5;
    for _ in 0..spike_count {
        let spike = commands
            .spawn((
                crate::windy::obstacles::Spike(SPIKE_LENGTH),
                Transform::from_translation(vec3(
                    (CONFIG.display.half_virtual_size.0 + SPIKE_LENGTH * 0.5) * -platforms.sign,
                    current_y,
                    -1.0,
                ))
                .with_rotation(Quat::from_rotation_z(
                    -std::f32::consts::FRAC_PI_2 * platforms.sign,
                )),
                canum_fx::util::DespawnCheck::new(event.entity),
            ))
            .id();
        commands.spawn((
            ChildOf(spike),
            enemy::movements::Displacement {
                displace: vec2(SPIKE_LENGTH * platforms.sign, 0.0),
                curve: |x| x,
                duration: Duration::from_secs_f32(0.2),
                notify: None,
            },
        ));
        platforms.child.push(spike);
        current_y -= SPIKE_LENGTH;
    }
    commands.spawn(Sound::new("Windy_SpikeOut").with_volume_add(-2.0));

    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(rand_normal(5.5, 0.35)),
        occupies: occupies![("Platforms", rand_normal(8.5, 1.0).min(10.0))],
    });
    if rand_bool(0.5) {
        commands.trigger(BehaveQueue::new(event.entity, "Ruler_BigLaser"));
    }
}

#[derive(Event)]
struct PlatformsDespawnChild(Vec<Entity>);
impl PlatformsDespawnChild {
    fn new(entities: Vec<Entity>) -> Self {
        Self(entities)
    }
}

#[derive(Component, Default)]
#[require(SessionOnly, projectile::RemoveOutOfBounds {distance_scale: 1.2})]
struct PlatformsBar;

fn platforms_spawn(
    mut q_platforms: Query<&mut Platforms>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for mut platforms in q_platforms.iter_mut() {
        let sign = platforms.sign;
        if platforms.order.is_empty() && !platforms.child.is_empty() {
            for spike in platforms.child.iter() {
                commands.spawn((
                    ChildOf(*spike),
                    enemy::movements::Displacement {
                        displace: vec2(-SPIKE_LENGTH * sign, 0.0),
                        curve: |x| x,
                        duration: Duration::from_secs_f32(0.2),
                        notify: None,
                    },
                ));
            }
            let entities = std::mem::take(&mut platforms.child);
            canum_fx::wait_then_trigger!(use commands, PlatformsDespawnChild, Vec::<Entity>, entities, 0.25);
            continue;
        }
        if platforms.interval.tick(time.delta()).just_finished() {
            let Some(y) = platforms.order.pop() else {
                continue;
            };
            let length = platforms.length * 1.2;
            let height = length * 3.0 / 20.0;
            commands.spawn((
                PlatformsBar,
                Animation::new("Wcat_Bar", vec2(length, height)),
                Collider::rectangle(length, height),
                Transform::from_translation(vec3(
                    (CONFIG.display.half_virtual_size.0 + height * 0.5) * platforms.sign,
                    y,
                    2.0,
                ))
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                LinearVelocity(vec2(-450.0 * platforms.sign, 0.0)),
                RigidBody::Kinematic,
                health::CollidePlayerOnly,
            ));
        }
    }
}

fn platforms_bar_accelerate(
    mut q_bar: Query<&mut LinearVelocity, With<PlatformsBar>>,
    time: Res<Time>,
) {
    const ACCELERATION: f32 = 300.0;
    q_bar.par_iter_mut().for_each(|mut linear_velocity| {
        linear_velocity.x += linear_velocity.x.signum() * ACCELERATION * time.delta_secs();
    });
}

fn platforms_despawn_child(event: On<PlatformsDespawnChild>, mut commands: Commands) {
    for child in event.0.iter() {
        commands.entity(*child).try_despawn();
    }
}

#[derive(Component, Default)]
#[require(Behavior::new("Ruler_BigLasers", 0.1, ["Main", "BigLasers"]))]
struct BigLasers;

fn big_lasers_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world.commands().entity(entity).observe(big_lasers_start);
}

const LASER_COUNT: usize = 6;
const HALF_LASER_COUNT: usize = LASER_COUNT / 2;
const BIG_LASER_GAP: f32 = 64.0;

#[derive(Event)]
struct BigLasersSpawn(bool);
impl BigLasersSpawn {
    fn new(subtracted: bool) -> Self {
        Self(subtracted)
    }
}

#[derive(Event)]
struct BigLasersDespawn(Vec<Entity>);
impl BigLasersDespawn {
    fn new(entities: Vec<Entity>) -> Self {
        Self(entities)
    }
}

fn big_lasers_start(event: On<BehaveStart>, mut commands: Commands) {
    let subtracted = rand_bool(0.6);
    let trigger = commands
        .spawn((OneSecondTrigger, Transform::default(), Visibility::Visible))
        .observe(OneSecondTrigger::observer)
        .id();
    for index in 0..HALF_LASER_COUNT {
        let x = if subtracted {
            CONFIG.display.half_virtual_size.0 - 32.0 - 64.0 * index as f32
        } else {
            32.0 + 64.0 * index as f32
        };
        commands.spawn((
            ChildOf(trigger),
            Animation::new("Windy_Warning", vec2(64.0, 64.0)),
            Transform::from_translation(vec3(x, 0.0, 15.0)),
        ));
        commands.spawn((
            ChildOf(trigger),
            Animation::new("Windy_Warning", vec2(64.0, 64.0)),
            Transform::from_translation(vec3(-x, 0.0, 15.0)),
        ));
    }
    commands.spawn(Sound::new("Windy_Warning"));

    canum_fx::wait_then_trigger!(use commands, BigLasersSpawn, bool, subtracted, 1.2);

    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(rand_normal(1.0, 0.2)),
        occupies: occupies![("BigLasers", 20.0)],
    });
}

fn big_lasers_spawn(event: On<BigLasersSpawn>, mut commands: Commands) {
    let mut entities = Vec::new();
    for index in 0..HALF_LASER_COUNT {
        let x = if event.0 {
            CONFIG.display.half_virtual_size.0 - BIG_LASER_GAP * (0.5 + index as f32)
        } else {
            BIG_LASER_GAP * (0.5 + index as f32)
        };
        let entity = commands
            .spawn((
                super::laser::RulerStrongLaser {
                    direction: vec2(0.0, -1.0),
                    double: false,
                },
                Transform::from_translation(vec3(x, CONFIG.display.half_virtual_size.1 - 1.0, 5.0)),
            ))
            .id();
        entities.push(entity);
        let entity = commands
            .spawn((
                super::laser::RulerStrongLaser {
                    direction: vec2(0.0, -1.0),
                    double: false,
                },
                Transform::from_translation(vec3(
                    -x,
                    CONFIG.display.half_virtual_size.1 - 1.0,
                    5.0,
                )),
            ))
            .id();
        entities.push(entity);
    }
    commands.spawn(Sound::new("Ruler_StrongLaser"));
    canum_fx::wait_then_trigger!(use commands, BigLasersDespawn, Vec::<Entity>, entities, 0.8);
}

fn big_lasers_despawn(event: On<BigLasersDespawn>, mut commands: Commands) {
    for entity in event.0.iter() {
        commands.entity(*entity).try_despawn();
    }
}
