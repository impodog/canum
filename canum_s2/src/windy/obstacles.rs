use super::*;

pub(super) struct ObstaclesPlugin;

impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().register_component_hooks::<Spike>().on_add(
            |mut world, HookContext { entity, .. }| {
                let Some(spike) = world.get::<Spike>(entity) else {
                    return;
                };
                let length = spike.0;
                let half_length = length * 0.5;
                world.commands().entity(entity).insert((
                    Animation::new("Windy_Spike", vec2(length, length)),
                    Collider::triangle(
                        vec2(-half_length, -half_length),
                        vec2(half_length, -half_length),
                        vec2(0.0, half_length),
                    ),
                ));
            },
        );
        app.world_mut()
            .register_component_hooks::<DestroyableObstacle>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(destroyable_damage);
            });
        app.add_systems(
            FixedUpdate,
            (update_tumble_weed_rotation, initialize_pole).run_if(in_state(WINDY_STATE.clone())),
        );
    }
}

#[derive(Component, Default)]
#[require(
    RigidBody::Kinematic,
    Collider,
    health::Friendly(false),
    health::ContactDamage { value: 50, projectile: false, order: consts::order::ENEMY_PROJ },
    projectile::NoCollideBoundary,
    projectile::RemoveOutOfBounds,
    movements::ForcedVelocity,
    Visibility,
)]
struct Obstacle;

#[derive(Component, Default)]
#[require(Obstacle, enemy::health::EnemyHealth)]
struct DestroyableObstacle;

fn destroyable_damage(
    event: On<health::Damage>,
    mut commands: Commands,
    q_health: Query<&enemy::health::EnemyHealth>,
) {
    let Ok(health) = q_health.get(event.entity) else {
        return;
    };
    if health.value <= 0 {
        commands.entity(event.entity).despawn();
    }
}

/// A spike of given side length.
#[derive(Component, Default)]
#[require(Obstacle, obstacle::BumpAway {direction: Dir2::from_xy_unchecked(0.0, 1.0), strength: 30.0})]
pub struct Spike(pub f32);

#[derive(Component, Default)]
#[require(
    DestroyableObstacle,
    RigidBody::Dynamic,
    Mass(1.0),
    movements::SpeedDecay(0.05),
    enemy::health::EnemyHealth::new(100),
    Collider::circle(Self::RADIUS),
    Animation::new("Windy_TumbleWeed", vec2(Self::RADIUS * 2.0, Self::RADIUS * 2.0)),
    wind::CanBeBlown
)]
pub struct TumbleWeed;

impl TumbleWeed {
    pub const RADIUS: f32 = 24.0;
}

fn update_tumble_weed_rotation(
    mut q_tumble_weed: Query<(&mut AngularVelocity, &LinearVelocity), With<TumbleWeed>>,
) {
    q_tumble_weed
        .par_iter_mut()
        .for_each(|(mut angular_velocity, linear_velocity)| {
            **angular_velocity = -linear_velocity.x / TumbleWeed::RADIUS;
        });
}

#[derive(Component, Default)]
#[require(Obstacle, wind::CanBeBlown)]
pub struct Pole {
    pub face_down: bool,
}
impl Pole {
    pub const SIZE: Vec2 = vec2(16.0, 32.0);
    pub const HALF_SIZE: Vec2 = vec2(Self::SIZE.x * 0.5, Self::SIZE.y * 0.5);
}

fn initialize_pole(
    q_pole: Query<(Entity, &Pole, &GlobalTransform), Added<Pole>>,
    mut commands: Commands,
) {
    for (entity, pole, global_transform) in q_pole.iter() {
        let sgn: f32 = if pole.face_down { -1.0 } else { 1.0 };
        let rotation = if pole.face_down {
            0.0
        } else {
            std::f32::consts::PI
        };
        let position = global_transform.translation().xy();
        let mut y = position.y;
        let mut accum_y = 0.0;
        while y < CONFIG.display.half_virtual_size.1 + Pole::HALF_SIZE.y
            && y > -CONFIG.display.half_virtual_size.1 - Pole::HALF_SIZE.y
        {
            commands.spawn((
                ChildOf(entity),
                Animation::new(
                    if accum_y == 0.0 {
                        "Windy_Pole_Top"
                    } else {
                        "Windy_Pole_Mid"
                    },
                    Pole::SIZE,
                ),
                Sprite {
                    flip_x: !pole.face_down,
                    ..default()
                },
                Transform::from_translation(vec3(0.0, accum_y * sgn, 0.0))
                    .with_rotation(Quat::from_rotation_z(rotation)),
            ));
            y += Pole::SIZE.y * sgn;
            accum_y += Pole::SIZE.y;
        }
        accum_y -= Pole::SIZE.y;
        commands.entity(entity).insert(Collider::compound(vec![(
            vec2(0.0, accum_y * 0.5 * sgn),
            Rotation::radians(0.0),
            Collider::rectangle(Pole::SIZE.x, Pole::SIZE.y + accum_y),
        )]));
    }
}
