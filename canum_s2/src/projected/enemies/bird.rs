use super::*;

pub(super) struct BirdPlugin;

impl Plugin for BirdPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<Bird>()
            .on_add(bird_hook);
        app.world_mut()
            .register_component_hooks::<RandomMoving>()
            .on_add(random_moving_hook);
        app.world_mut()
            .register_component_hooks::<ShootFeather>()
            .on_add(shoot_feather_hook);
    }
}

#[derive(Component, Default)]
#[require(
    ProjectedEnemy,
    Collider::rectangle(10.0, 20.0),
    Animation::new("Projected_Bird_Static", vec2(64.0, 64.0)),
    enemy::health::EnemyHealth::new(150),
    movements::AutoFlip::FLIP_RIGHT
)]
pub struct Bird;

fn bird_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world.commands().spawn((
        ChildOf(entity),
        enemy::health::EnemySensor,
        Collider::circle(26.0),
    ));
    world.commands().spawn((
        ChildOf(entity),
        BirdBehaviors,
        children![RandomMoving::default(), ShootFeather],
    ));
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
struct BirdBehaviors;

#[derive(Component, Default)]
#[require(Behavior::new("Projected_Bird_RandomMoving", 1.0, ["RandomMoving"]))]
struct RandomMoving {
    target: Option<Entity>,
}

fn random_moving_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world
        .commands()
        .entity(entity)
        .observe(random_moving_start)
        .observe(random_moving_end);
}

fn random_moving_start(
    event: On<BehaveStart>,
    mut commands: Commands,
    q_transform: Query<&GlobalTransform>,
    mut q_animation: Query<&mut Animation>,
    mut q_behavior: Query<&mut RandomMoving>,
) {
    let Ok(mut random_moving) = q_behavior.get_mut(event.entity) else {
        return;
    };
    random_moving.target = Some(event.target);

    let Ok(transform) = q_transform.get(event.entity) else {
        return;
    };
    let position = transform.translation().xy();
    let displace = if !CONFIG.display.screen_rect.contains(position) || rand::random_bool(0.2) {
        position.normalize_or(vec2(1.0, 0.0)) * -160.0
    } else {
        let perp = position.normalize_or(vec2(1.0, 0.0)).perp() * rand_sign();
        let direction = perp.rotate(Vec2::from_angle(rand_normal(0.0, 0.5)));
        direction * rand_normal(100.0, 20.0).clamp(80.0, 130.0)
    };
    commands.spawn((
        ChildOf(event.target),
        enemy::movements::Displacement {
            curve: |x| x,
            displace,
            duration: Duration::from_secs_f32(displace.length() / 85.0),
            notify: Some(event.entity),
        },
    ));

    let Ok(mut animation) = q_animation.get_mut(event.target) else {
        return;
    };
    animation.replace("Projected_Bird_Walking", false, None);
}

fn random_moving_end(
    event: On<enemy::movements::DisplacementComplete>,
    mut commands: Commands,
    mut q_animation: Query<&mut Animation>,
    q_behavior: Query<&RandomMoving>,
) {
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(rand_normal(3.0, 0.6)),
        occupies: occupies![],
    });

    let Ok(random_moving) = q_behavior.get(event.entity) else {
        return;
    };
    if let Some(target) = random_moving.target {
        let Ok(mut animation) = q_animation.get_mut(target) else {
            return;
        };
        animation.replace("Projected_Bird_Static", false, None);
    }
}

#[derive(Component, Default)]
#[require(
    Animation::new("Projected_Bird_Feather", Vec2::new(48.0, 48.0)),
    enemy::attack::EnemyProjectile,
    movements::ForcedVelocity,
    Collider::rectangle(20.0, 10.0),
    Mass(1.0)
)]
struct BirdFeather;

#[derive(Component, Default)]
#[require(Behavior::new("Projected_Bird_ShootFeather", 0.5, ["ShootFeather"]))]
struct ShootFeather;

fn shoot_feather_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world.commands().entity(entity).observe(shoot_feather);
}

fn shoot_feather(
    event: On<BehaveStart>,
    mut commands: Commands,
    q_transform: Query<&GlobalTransform>,
    player: Option<Res<player::PrimaryPlayer>>,
) {
    let Some(player) = player else {
        return;
    };
    let Ok([player_transform, transform]) = q_transform.get_many([player.0, event.entity]) else {
        return;
    };
    let player_position = player_transform.translation().xy();
    let position = transform.translation().xy();
    let direction = (player_position - position).normalize_or(vec2(1.0, 0.0));

    let transform = Transform::from_translation(vec3(position.x, position.y, 0.0))
        .with_rotation(Quat::from_rotation_z(direction.to_angle()));
    commands.spawn((BirdFeather, transform, LinearVelocity(direction * 200.0)));
    commands.spawn(Sound::new("Projected_Bird_Chirp"));
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(1.0),
        occupies: occupies![("ShootFeather", rand_normal(9.0, 1.0))],
    });
}
