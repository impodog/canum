use super::*;
use enemy::behavior::*;

pub(super) struct ObstaclesPlugin;

impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            init_bar.run_if(in_state(RUNWAY_STATE.clone())),
        );
        app.world_mut()
            .register_component_hooks::<ObstacleBehaviors>()
            .on_add(|mut world, HookContext { entity, .. }| {
                let mut commands = world.commands();
                commands
                    .spawn((ChildOf(entity), RotatingBar))
                    .observe(rotating_bar);
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
        occupies: occupies![("Slow", 3.0)],
    });
}
