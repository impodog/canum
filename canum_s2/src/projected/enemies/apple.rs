use super::*;

pub(super) struct ApplePlugin;

impl Plugin for ApplePlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<Apple>()
            .on_add(apple_hook);
        app.world_mut()
            .register_component_hooks::<RandomMove>()
            .on_add(random_move_hook);
        app.world_mut()
            .register_component_hooks::<ThrowSlice>()
            .on_add(throw_slice_hook);
        app.add_systems(FixedUpdate, slice_accelerate.in_set(ProjectedSet));
    }
}

/// This mini boss is meant to recall the first boss of the game
#[derive(Component, Default)]
#[require(
    ProjectedEnemy,
    Collider::circle(15.0),
    Animation::new("Projected_Apple", vec2(64.0, 64.0)),
    enemy::health::EnemyHealth::new(550),
    movements::AutoFlip::FLIP_RIGHT
)]
pub struct Apple;

fn apple_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world.commands().spawn((
        ChildOf(entity),
        AppleBehaviors,
        children![RandomMove, ThrowSlice],
    ));
    world.commands().spawn((
        ChildOf(entity),
        enemy::health::EnemySensor,
        Collider::circle(30.0),
    ));
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
struct AppleBehaviors;

#[derive(Component, Default)]
#[require(Behavior::new("Projected_Apple_RandomMove", 1.0, ["RandomMove"]))]
struct RandomMove;

fn random_move_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world
        .commands()
        .entity(entity)
        .observe(random_move)
        .observe(random_move_end);
}

fn random_move(
    event: On<BehaveStart>,
    mut commands: Commands,
    q_transform: Query<&GlobalTransform>,
) {
    let Ok(transform) = q_transform.get(event.entity) else {
        return;
    };
    let position = transform.translation().xy();
    let displace = if CONFIG.display.screen_rect.contains(position) && rand::random_bool(0.9) {
        rand_normal(80.0, 10.0) * Vec2::from_angle(rand::random_range(0.0..std::f32::consts::TAU))
    } else {
        let direction = (-position.normalize_or(vec2(1.0, 0.0)))
            .rotate(Vec2::from_angle(rand_normal(0.0, 0.08)));
        direction * rand_normal(100.0, 7.0)
    };
    commands.spawn((
        ChildOf(event.target),
        enemy::movements::Displacement {
            curve: |x| QuadraticInOutCurve.sample(x).unwrap(),
            displace,
            duration: Duration::from_secs_f32(displace.length() / 100.0),
            notify: Some(event.entity),
        },
    ));
}
fn random_move_end(event: On<enemy::movements::DisplacementComplete>, mut commands: Commands) {
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(0.1),
        occupies: occupies![("RandomMove", 2.0)],
    });
}

#[derive(Component, Default)]
#[require(
    enemy::attack::EnemyProjectile,
    Animation::new("Projected_Apple_Slice", vec2(48.0, 48.0),),
    Collider::rectangle(24.0, 10.0)
)]
struct AppleSlice {
    /// Direction of acceleration
    direction: Vec2,
}

#[derive(Component, Default)]
#[require(Behavior::new("Projected_Apple_ThrowSlice", 1.0, ["ThrowSlice"]))]
struct ThrowSlice;

fn throw_slice_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world.commands().entity(entity).observe(throw_slice);
}

const SLICE_ACCELERATION: f32 = 400.0;
const SLICE_INITIAL_SPEED: f32 = 100.0;

fn throw_slice(
    event: On<BehaveStart>,
    mut commands: Commands,
    q_transform: Query<&GlobalTransform>,
    player: Option<Res<player::PrimaryPlayer>>,
) {
    let Some(player) = player else { return };
    let Ok(player_transform) = q_transform.get(player.0) else {
        return;
    };
    let player_position = player_transform.translation().xy();
    let Ok(transform) = q_transform.get(event.entity) else {
        return;
    };
    let position = transform.translation().xy();
    let direction = (player_position - position).normalize_or(vec2(1.0, 0.0));
    commands.spawn((
        AppleSlice { direction },
        Transform::from_translation(vec3(position.x, position.y, 0.05)),
        LinearVelocity(-direction * SLICE_INITIAL_SPEED),
    ));
    commands.spawn(Sound::new("Apple_Swoosh"));
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(0.5),
        occupies: occupies![("ThrowSlice", rand_normal(6.0, 0.5))],
    });
}

fn slice_accelerate(mut q_slice: Query<(&AppleSlice, &mut LinearVelocity)>, time: Res<Time>) {
    q_slice
        .par_iter_mut()
        .for_each(|(apple_slice, mut linear_velocity)| {
            **linear_velocity += apple_slice.direction * (SLICE_ACCELERATION * time.delta_secs());
        });
}
