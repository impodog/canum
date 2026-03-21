use crate::prelude::*;

use std::collections::BTreeMap;

pub(super) struct HealthPlugin;
impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPostUpdate,
            (work_invincibility_timer, add_observer_to_integer_health),
        );
    }
}

/// Sends an event to damage the entity, and what exactly to do depends on health implementation.
#[derive(EntityEvent, Debug)]
pub struct Damage {
    pub entity: Entity,
    /// The amount of damage. This doesn't do much for player health bars.
    pub value: i32,
    /// Which order(strength) this damage has. High order damages may override low-order shields and invincibility effects.
    pub order: u8,
}

/// Stores the layers of shields added to the entity.
/// Shields with larger priority will be used first.
#[derive(Component, Default, Debug, Deref, DerefMut)]
pub struct Shields(BTreeMap<u8, i32>);
impl Shields {
    /// Try to absorb as much damage as possible using shield, and then return the remaining amount.
    pub fn take_damage(&mut self, mut damage: i32) -> i32 {
        while damage > 0
            && let Some(mut entry) = self.last_entry()
        {
            let value = *entry.get();
            if value > damage {
                *entry.get_mut() -= damage;
                return 0;
            } else {
                damage -= value;
                entry.remove();
            }
        }
        damage
    }
}

/// As a child, times the invincibility effect, and removes the linked shield when timer is done.
#[derive(Component, Default, Debug, Deref, DerefMut)]
pub struct InvincibilityTimer {
    #[deref]
    pub timer: Timer,
    pub linked_shield: u8,
}
impl InvincibilityTimer {
    pub fn new(seconds: f32, linked_shield: u8) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
            linked_shield,
        }
    }
}

fn work_invincibility_timer(
    mut commands: Commands,
    mut q_timer: Query<(Entity, Mut<InvincibilityTimer>, &ChildOf)>,
    time: Res<Time>,
    mut q_shields: Query<&mut Shields>,
) -> Result<()> {
    for (entity, mut timer, parent) in q_timer.iter_mut() {
        if timer.is_added() {
            q_shields
                .get_mut(parent.0)?
                .insert(timer.linked_shield, i32::MAX);
        }
        timer.tick(time.delta());
        if timer.just_finished() {
            q_shields.get_mut(parent.0)?.remove(&timer.linked_shield);
            commands.entity(entity).despawn();
        }
    }
    Ok(())
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
            count: 7,
            invinc_order: 200,
            invinc_time: 0.5,
        }
    }
}
fn add_observer_to_integer_health(
    mut commands: Commands,
    q_health: Query<Entity, Added<IntegerHealth>>,
) {
    for entity in q_health.iter() {
        commands.entity(entity).observe(integer_take_damage);
    }
}
fn integer_take_damage(
    event: On<Damage>,
    mut commands: Commands,
    mut q_health: Query<(&mut IntegerHealth, &mut Shields)>,
) -> Result<()> {
    let (mut health, mut shields) = q_health.get_mut(event.entity)?;
    let damage = shields.take_damage(event.value);
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
    Ok(())
}
