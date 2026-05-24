use super::*;
use enemy::behavior::*;

pub(super) struct ObstaclesPlugin;

impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (init_bar, init_spike, init_tree, init_box).run_if(in_state(RUNWAY_STATE.clone())),
        );
        app.world_mut()
            .register_component_hooks::<ObstacleBehaviors>()
            .on_add(|mut world, HookContext { entity, .. }| {
                let mut commands = world.commands();
                commands
                    .spawn((ChildOf(entity), RotatingBar))
                    .observe(rotating_bar);
                commands
                    .spawn((ChildOf(entity), StaircaseBars))
                    .observe(staircase_bars);
                commands
                    .spawn((ChildOf(entity), SpawnTree))
                    .observe(spawn_tree);
                commands
                    .spawn((ChildOf(entity), SpawnBoxAndBar))
                    .observe(spawn_box_and_bar);
            });
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager)]
pub struct ObstacleBehaviors;

// --- Implement specific obstacle shapes.

#[derive(Component, Default)]
#[require(
    RigidBody::Kinematic,
    Collider,
    health::Friendly(false),
    health::ContactDamage { value: 150, projectile: false, order: consts::order::ENEMY_PROJ },
    projectile::NoCollideBoundary,
    projectile::RemoveOutOfBounds,
    movements::ForcedVelocity,
    Visibility,
)]
struct Obstacle;

#[derive(Component)]
#[require(Obstacle)]
struct Bar {
    length: f32,
}
impl Default for Bar {
    fn default() -> Self {
        Self {
            length: CONFIG.display.half_virtual_size.0,
        }
    }
}
fn init_bar(mut q_bar: Query<(Entity, &mut Collider, &Bar), Added<Bar>>, mut commands: Commands) {
    const BAR_HEIGHT: f32 = 10.0;
    for (entity, mut collider, bar) in q_bar.iter_mut() {
        *collider = Collider::rectangle(bar.length, BAR_HEIGHT);
        let half_length = bar.length * 0.5;
        commands.spawn((
            ChildOf(entity),
            Animation::new("Runway_Bar_Left", Vec2::new(BAR_HEIGHT, BAR_HEIGHT)),
            Transform::from_translation(vec3(-half_length + BAR_HEIGHT * 0.5, 0.0, 0.0)),
        ));
        commands.spawn((
            ChildOf(entity),
            Animation::new("Runway_Bar_Right", Vec2::new(BAR_HEIGHT, BAR_HEIGHT)),
            Transform::from_translation(vec3(-half_length + BAR_HEIGHT * 0.5, 0.0, 0.01)),
        ));
        let remaining_length = bar.length - BAR_HEIGHT * 2.0;
        if remaining_length > 0.0 {
            commands.spawn((
                ChildOf(entity),
                Animation::new("Runway_Bar_Middle", Vec2::new(remaining_length, BAR_HEIGHT)),
            ));
        }
    }
}

// -- Implement obstacle spawning logic

#[derive(Component, Default)]
#[require(Behavior::new("Runway_RotatingBar", 1.0, ["Slow"]))]
struct RotatingBar;

fn rotating_bar(event: On<BehaveStart>, mut commands: Commands) {
    let start_sign = rand_sign();
    for x_sign in [1.0, -1.0] {
        let angular_velocity = start_sign * x_sign * rand_normal(1.77, 0.2);
        commands.spawn((
            Bar {
                length: CONFIG.display.screen_size.x * 0.25,
            },
            Transform::from_translation(vec3(
                CONFIG.display.half_virtual_size.0 * (0.5 + x_sign * 0.25),
                -CONFIG.display.screen_size.y + 50.0,
                1.0,
            )),
            AngularVelocity(angular_velocity),
        ));
    }
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(1.0),
        occupies: occupies![("Slow", rand_normal(5.5, 0.7))],
    });
}

#[derive(Component, Default)]
#[require(Behavior::new("Runway_StaircaseBars", 0.5, ["Slow"]))]
struct StaircaseBars;

fn staircase_bars(event: On<BehaveStart>, mut commands: Commands) {
    let begin_sign = rand::random_bool(0.5);
    let number = rand::random_range(3..5);
    let length = rand_normal(CONFIG.display.half_virtual_size.0 * 0.6, 20.0);
    let spacing = rand_normal(250.0, 25.0);
    for index in 0..number {
        let sign: f32 = if begin_sign ^ ((index & 1) == 0) {
            1.0
        } else {
            -1.0
        };
        commands.spawn((
            Bar { length },
            projectile::RemoveOutOfBounds {
                distance_scale: 0.3,
            },
            Transform::from_translation(vec3(
                CONFIG.display.half_virtual_size.0 * 0.5
                    + (CONFIG.display.half_virtual_size.0 - length) * 0.5 * sign,
                -CONFIG.display.half_virtual_size.1 - spacing * index as f32,
                1.0,
            )),
        ));
    }
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(2.0),
        occupies: occupies![("Slow", rand_normal(9.0, 0.5))],
    });
}

#[derive(Component, Default)]
#[require(Obstacle)]
pub struct Spike {
    pub direction: f32,
}
impl Spike {
    pub const SPIKE_EXTENTS: Vec2 = vec2(32.0, 32.0);
}

fn init_spike(
    mut q_spike: Query<(Entity, &mut Transform, &mut Collider, &Spike), Added<Spike>>,
    mut commands: Commands,
) {
    const HALF_FACTOR: f32 = 0.45;
    for (entity, mut transform, mut collider, spike) in q_spike.iter_mut() {
        commands
            .entity(entity)
            .insert(Animation::new("Runway_Spike", Spike::SPIKE_EXTENTS));
        transform.rotation = Quat::from_rotation_z(spike.direction - std::f32::consts::FRAC_PI_2);
        *collider = Collider::triangle_unchecked(
            vec2(
                -Spike::SPIKE_EXTENTS.x * HALF_FACTOR,
                -Spike::SPIKE_EXTENTS.y * HALF_FACTOR,
            ),
            vec2(
                Spike::SPIKE_EXTENTS.x * HALF_FACTOR,
                -Spike::SPIKE_EXTENTS.y * HALF_FACTOR,
            ),
            vec2(0.0, Spike::SPIKE_EXTENTS.y * 0.5),
        );
    }
}

#[derive(Component)]
#[require(Obstacle, Animation::new("Runway_Tree", vec2(64.0, 128.0)))]
struct Tree;

fn init_tree(mut q_tree: Query<&mut Collider, Added<Tree>>) {
    const COLLIDER_SIZE: Vec2 = vec2(32.0, 10.0);
    for mut collider in q_tree.iter_mut() {
        *collider = Collider::compound(vec![(
            vec2(0.0, -64.0 + COLLIDER_SIZE.y),
            0.0,
            Collider::rectangle(COLLIDER_SIZE.x, COLLIDER_SIZE.y),
        )]);
    }
}

#[derive(Component, Default)]
#[require(Behavior::new("Runway_SpawnTree", 0.4, ["Tree"]))]
struct SpawnTree;

fn spawn_tree(event: On<BehaveStart>, mut commands: Commands) {
    let x = rand::random_range(10.0..CONFIG.display.half_virtual_size.0 - 10.0);
    commands.spawn((
        Tree,
        Transform::from_translation(Vec3::new(
            x,
            -128.0 - CONFIG.display.half_virtual_size.1,
            25.37,
        )),
    ));
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(0.1),
        occupies: occupies![("Tree", rand_normal(3.0, 2.0).clamp(0.5, 3.0))],
    });
}

#[derive(Component)]
#[require(Obstacle, Animation, enemy::health::EnemyHealth::new(Self::HEALTH))]
struct BreakableBox;
impl BreakableBox {
    const SIZE: Vec2 = vec2(60.0, 60.0);
    const HEALTH: i32 = 100;
}

fn init_box(
    mut q_box: Query<(Entity, &mut Animation, &mut Collider), Added<BreakableBox>>,
    mut commands: Commands,
) {
    for (entity, mut animation, mut collider) in q_box.iter_mut() {
        *animation = Animation::new("Runway_Box", BreakableBox::SIZE);
        animation.set_pause(1);
        *collider = Collider::rectangle(BreakableBox::SIZE.x * 0.95, BreakableBox::SIZE.y * 0.95);
        commands
            .entity(entity)
            .observe(box_breaks)
            .observe(box_damaged);
    }
}

fn box_damaged(event: On<health::Damage>, q_box: Query<(&enemy::health::EnemyHealth, &Animation)>) {
    let Ok((health, animation)) = q_box.get(event.entity) else {
        return;
    };
    if health.value > 0 {
        let index = ((BreakableBox::HEALTH - health.value) * 3 / BreakableBox::HEALTH) as usize;
        animation.set_pause(index + 1);
    }
}

fn box_breaks(
    event: On<enemy::health::EnemyDefeated>,
    q_box: Query<&Animation>,
    mut commands: Commands,
) {
    let Ok(animation) = q_box.get(event.entity) else {
        return;
    };
    animation.set_pause(0);
    animation.set_inform(AnimationInform {
        entity: event.entity,
        index: vec![0],
    });
    commands
        .entity(event.entity)
        .remove::<Collider>()
        .observe(box_break_animation_done);
}

fn box_break_animation_done(event: On<AnimationComplete>, mut commands: Commands) {
    commands.entity(event.entity).despawn();
}

#[derive(Component)]
#[require(Behavior::new("Runway_SpawnBoxAndBar", 0.5, ["Slow"]))]
struct SpawnBoxAndBar;

fn spawn_box_and_bar(event: On<BehaveStart>, mut commands: Commands) {
    let start_y = -CONFIG.display.half_virtual_size.1 - BreakableBox::SIZE.y * 0.5;
    let box_position = rand::random_range(
        BreakableBox::SIZE.x * 2.0
            ..=CONFIG.display.half_virtual_size.0 - BreakableBox::SIZE.x * 2.0,
    );
    commands.spawn((
        BreakableBox,
        Transform::from_translation(vec3(box_position, start_y, 0.1)),
    ));
    let left_length = box_position - BreakableBox::SIZE.x * 0.5;
    let left_length = (left_length - 10.0).max(10.0);
    let right_length =
        (CONFIG.display.half_virtual_size.0 - box_position) - BreakableBox::SIZE.x * 0.5;
    let right_length = (right_length - 10.0).max(100.0);
    commands.spawn((
        Bar {
            length: left_length,
        },
        Transform::from_translation(vec3(left_length * 0.5, start_y, 0.0)),
    ));
    commands.spawn((
        Bar {
            length: right_length,
        },
        Transform::from_translation(vec3(
            CONFIG.display.half_virtual_size.0 - right_length * 0.5,
            start_y,
            0.0,
        )),
    ));
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(0.5),
        occupies: occupies![("Slow", rand_normal(10.0, 0.6))],
    });
}
