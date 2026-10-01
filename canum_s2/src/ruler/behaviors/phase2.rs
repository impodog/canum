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
        app.add_systems(FixedUpdate, horiz_laser_accelerate.in_set(RulerSet));
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct RulerPhase2;

fn ruler_phase2_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let mut commands = world.commands();
    commands.spawn((ChildOf(entity), HorizLaserAcc));
    commands.spawn((ChildOf(entity), ManyBallsFall));
    canum_fx::session_observers!(
        commands,
        horiz_laser_observe,
        horiz_laser_warning,
        many_balls_fall_action
    );
}

#[derive(Component, Default)]
#[require(Behavior::new("Ruler_HorizLaserAcc", 1.0, ["Horiz", "HorizLaserAcc"]))]
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
        cooldown: Duration::from_secs_f32(1.5),
        occupies: occupies![("Horiz", rand_normal(6.0, 0.32))],
    });
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
    commands.spawn((
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
    ));
}
fn horiz_laser_accelerate(mut q_laser: Query<(&mut LinearVelocity, &HorizLaser)>, time: Res<Time>) {
    for (mut linear_velocity, horiz_laser) in q_laser.iter_mut() {
        linear_velocity.x += time.delta_secs() * horiz_laser.acc;
    }
}

#[derive(Component, Debug)]
#[require(Behavior::new("Ruler_ManyBallsFall", 0.8, ["Horiz", "Verti", "ManyBallsFall"]))]
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

    let delta = rand_normal(0.0, 0.25);
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(3.0),
        occupies: occupies![("Verti", 6.0 + delta), ("Horiz", 6.0 - delta)],
    });
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
