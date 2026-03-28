use bevy::ecs::lifecycle::HookContext;

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
    }
}

/// Basic player health bar, allowing to take only integer number of damage.
#[derive(Component, Debug)]
#[require(Shields)]
pub struct IntegerHealth {
    pub count: i32,
    pub invinc_order: u8,
    pub invinc_time: f32,
}
impl Default for IntegerHealth {
    fn default() -> Self {
        Self {
            count: 6,
            invinc_order: 200,
            invinc_time: 0.5,
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
    health.count = health.count.saturating_sub(1);
    commands
        .entity(event.entity)
        .insert(children![InvincibilityTimer::new(
            health.invinc_time,
            health.invinc_order
        )]);
    commands.spawn(canum_res::sound::Sound::new("BasicHp_Damage"));
    Ok(())
}
