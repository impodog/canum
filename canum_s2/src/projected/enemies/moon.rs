use super::*;

pub(super) struct MoonPlugin;

impl Plugin for MoonPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<Moon>()
            .on_add(moon_hook);
        app.add_systems(
            FixedUpdate,
            (star_rotate, moon_move_closer).in_set(ProjectedSet),
        );
    }
}

#[derive(Component, Default)]
#[require(
    ProjectedEnemy,
    Collider::circle(10.0),
    Animation::new("Projected_Moon", vec2(48.0, 48.0)),
    enemy::health::EnemyHealth::new(300),
    movements::AutoFlip::FLIP_RIGHT
)]
pub struct Moon;

fn moon_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world.commands().spawn((
        ChildOf(entity),
        enemy::health::EnemySensor,
        Collider::circle(24.0),
    ));
    let mut current = rand::random_range(0.0..std::f32::consts::TAU);
    for _ in 0..STAR_COUNT {
        let position = Vec2::from_angle(current) * STAR_RADIUS;
        world.commands().spawn((
            ChildOf(entity),
            Star,
            Transform::from_translation(vec3(position.x, position.y, -0.1)),
        ));
        current += STAR_ANGLE_DIFF;
    }
}

#[derive(Component, Default)]
#[require(
    enemy::attack::EnemyProjectile,
    Animation::new("Projected_Star", vec2(32.0, 32.0)),
    Collider::circle(7.5)
)]
struct Star;

const STAR_ANGULAR_VELOCITY: f32 = std::f32::consts::PI;
const STAR_RADIUS: f32 = 70.0;
const STAR_COUNT: usize = 3;
const STAR_ANGLE_DIFF: f32 = std::f32::consts::FRAC_PI_3 * 2.0;

fn star_rotate(mut q_star: Query<&mut Transform, With<Star>>, time: Res<Time>) {
    q_star.par_iter_mut().for_each(|mut transform| {
        let xy = transform.translation.xy();
        let new_xy = xy.rotate(Vec2::from_angle(STAR_ANGULAR_VELOCITY * time.delta_secs()));
        transform.translation.x = new_xy.x;
        transform.translation.y = new_xy.y;
    });
}

fn moon_move_closer(
    mut q_moon: Query<(&GlobalTransform, &mut movements::ForcedVelocity)>,
    q_transform: Query<&GlobalTransform>,
    player: Option<Res<player::PrimaryPlayer>>,
) {
    let Some(player) = player else {
        return;
    };
    let Ok(player_transform) = q_transform.get(player.0) else {
        return;
    };
    let player_position = player_transform.translation().xy();
    q_moon
        .par_iter_mut()
        .for_each(|(global_transform, mut velocity)| {
            let position = global_transform.translation().xy();
            let diff = player_position - position;
            let direction = diff.normalize_or_zero();
            **velocity = direction * ((diff.length() * 0.3).clamp(10.0, 70.0))
        });
}
