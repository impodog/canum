use crate::health::*;
use crate::prelude::*;

pub(super) struct PlayerHealthPlugin;

impl Plugin for PlayerHealthPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<IntegerHealth>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(integer_take_damage);
            });
        app.add_systems(FixedFirst, refresh_integer_health);
    }
}

/// Sent to player itself by health, if the player is actually hit without shield protection.
#[derive(EntityEvent, Debug)]
pub struct ActuallyHit {
    pub entity: Entity,
}

/// Basic player health bar, allowing to take only integer number of damage.
#[derive(Component, Debug)]
#[require(HealthBar)]
pub struct IntegerHealth {
    pub count: i32,
    pub invinc_order: u8,
    pub invinc_time: f32,
    /// Prevents multiple hits.
    pub frame_taken_damage: bool,
}
impl Default for IntegerHealth {
    fn default() -> Self {
        Self {
            count: 6,
            invinc_order: consts::order::HEALTH_INVINC,
            invinc_time: 1.0,
            frame_taken_damage: false,
        }
    }
}
fn integer_take_damage(
    event: On<Damage>,
    mut commands: Commands,
    mut q_health: Query<(&mut IntegerHealth, &mut Shields)>,
) -> Result<()> {
    let (mut health, mut shields) = q_health.get_mut(event.entity)?;
    let damage = shields.take_damage(event.value, event.order);
    if damage <= 0 {
        return Ok(());
    }
    if health.frame_taken_damage {
        return Ok(());
    }
    health.frame_taken_damage = true;
    health.count = health.count.saturating_sub(1);
    commands.spawn(canum_res::sound::Sound::new("BasicHp_Damage"));
    commands.trigger(ActuallyHit {
        entity: event.entity,
    });
    if health.count > 0 {
        commands.spawn((
            ChildOf(event.entity),
            InvincibilityTimer::new(health.invinc_time, health.invinc_order),
        ));
        commands.spawn((
            ChildOf(event.entity),
            canum_fx::splash::Splash {
                color: Color::linear_rgba(0.0, 1.0, 1.0, 0.3),
                duration: std::time::Duration::from_secs_f32(0.2),
                number: 10,
            },
        ));
    } else {
        commands.trigger(super::failure::PlayerFail);
    }

    Ok(())
}

fn refresh_integer_health(mut q_health: Query<&mut IntegerHealth>) {
    for mut health in q_health.iter_mut() {
        health.frame_taken_damage = false;
    }
}
