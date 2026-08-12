use super::*;
use enemy::behavior::*;

pub(super) struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<SpawnTumbleWeed>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(spawn_tumble_weed);
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
) {
    const RADIUS: f32 = obstacles::TumbleWeed::RADIUS;
    if let Some(wind) = wind {
        let sgn = wind.target_velocity.x.signum();
        commands.spawn((
            obstacles::TumbleWeed,
            Transform::from_translation(vec3(
                (CONFIG.display.half_virtual_size.0 + RADIUS) * -sgn,
                rand::random_range(
                    -CONFIG.display.half_virtual_size.1 + RADIUS
                        ..CONFIG.display.half_virtual_size.1 - RADIUS,
                ),
                2.1,
            )),
        ));
    }
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(0.1),
        occupies: occupies![("SpawnTumbleWeed", rand_normal(0.5, 0.2).clamp(0.15, 1.0))],
    });
}
