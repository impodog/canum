use bevy::ecs::system::SystemParam;

use crate::prelude::*;

use std::collections::{BTreeMap, BTreeSet};

pub(super) struct HealthPlugin;
impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, (work_invincibility_timer,));
        app.add_systems(FixedPreUpdate, deal_contact_damage);
        app.add_systems(FixedPostUpdate, init_contact_damage);
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

/// Marks this entity a health bar of its parent.
#[derive(Component, Default, Debug, Clone, Copy)]
#[require(Shields)]
pub struct HealthBar;

/// Stores the layers of shields added to the entity.
/// Shields with larger priority will be used first.
#[derive(Component, Default, Debug, Deref, DerefMut)]
pub struct Shields(BTreeMap<u8, i32>);
impl Shields {
    /// Try to absorb as much damage as possible using shield, and then return the remaining amount.
    pub fn take_damage(&mut self, mut damage: i32, order: u8) -> i32 {
        while damage > 0
            && let Some(mut entry) = self.last_entry()
            && *entry.key() > order
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

/// Marks if an entity is friendly to player.
#[derive(Component, Default, Deref, DerefMut)]
#[require(
    ActiveCollisionHooks::FILTER_PAIRS,
    CollisionEventsEnabled,
    crate::SessionOnly
)]
pub struct Friendly(pub bool);

/// Marks to collide only with the player.
#[derive(Component, Default)]
pub struct CollidePlayerOnly;

/// Insert this component to temporarily disable collision from opposite friendliness.
#[derive(Component, Default)]
pub struct DisableOpposingCollision;

impl Friendly {
    pub const FRIENDLY: Friendly = Friendly(true);
    pub const UNFRIENDLY: Friendly = Friendly(false);
}

/// This prevents friendly objects from interacting with each other.
#[derive(SystemParam)]
pub struct PhysicsHooks<'w, 's> {
    q_friendly: Query<'w, 's, &'static Friendly>,
    q_disable_opposite: Query<'w, 's, &'static DisableOpposingCollision>,
    q_no: Query<'w, 's, &'static crate::projectile::NoCollideBoundary>,
    q_boundary: Query<'w, 's, &'static crate::setup::Boundaries>,
    q_projectile: Query<'w, 's, &'static crate::projectile::Projectile>,
    q_oneway: Query<'w, 's, &'static crate::obstacle::OnewayCollision>,
    q_velocity: Query<'w, 's, &'static LinearVelocity>,
    q_player_only: Query<'w, 's, &'static CollidePlayerOnly>,
    q_player: Query<'w, 's, &'static crate::player::Player>,
}
impl<'w, 's> CollisionHooks for PhysicsHooks<'w, 's> {
    fn filter_pairs(&self, collider1: Entity, collider2: Entity, _commands: &mut Commands) -> bool {
        if self.q_projectile.get_many([collider1, collider2]).is_ok() {
            return false;
        }
        #[allow(clippy::collapsible_if)]
        if let Ok([friendly1, friendly2]) = self.q_friendly.get_many([collider1, collider2]) {
            if self.q_disable_opposite.get(collider1).is_ok()
                || self.q_disable_opposite.get(collider2).is_ok()
            {
                return false;
            }
            if friendly1.0 == friendly2.0 {
                return false;
            }
        }
        if self.q_no.get(collider1).is_ok() && self.q_boundary.get(collider2).is_ok() {
            return false;
        }
        if self.q_boundary.get(collider1).is_ok() && self.q_no.get(collider2).is_ok() {
            return false;
        }
        if let Ok([velocity1, velocity2]) = self.q_velocity.get_many([collider1, collider2]) {
            let relative = velocity2.0 - velocity1.0;
            if let Ok(oneway) = self.q_oneway.get(collider1)
                && !oneway.test_velocity(relative)
            {
                return false;
            }
            if let Ok(oneway) = self.q_oneway.get(collider2)
                && oneway.test_velocity(relative)
            {
                return false;
            }
        }
        if self.q_player_only.get(collider1).is_ok() && self.q_player.get(collider2).is_err() {
            return false;
        }
        if self.q_player_only.get(collider2).is_ok() && self.q_player.get(collider1).is_err() {
            return false;
        }
        true
    }
}

/// Marks an entity to deal contact damage. This can either be used on enemies or projectiles.
#[derive(Component, Debug, Default)]
#[require(Friendly, CollidingEntities)]
pub struct ContactDamage {
    pub value: i32,
    /// Prevents multiple hits.
    pub projectile: bool,
    pub order: u8,
}
/// Stores the projectile's contact history, preventing multiple hits.
#[derive(Component, Debug, Deref, DerefMut, Default)]
pub struct ProjectileContacted(BTreeSet<Entity>);

fn init_contact_damage(
    commands: ParallelCommands,
    q_added: Query<(Entity, &ContactDamage), Added<ContactDamage>>,
) {
    q_added.par_iter().for_each(|(entity, contact_damage)| {
        if contact_damage.projectile {
            commands.command_scope(|mut commands| {
                commands
                    .entity(entity)
                    .insert(ProjectileContacted::default());
            });
        }
    });
}

fn deal_contact_damage(
    commands: ParallelCommands,
    mut q_contact_damage: Query<(
        &ContactDamage,
        &CollidingEntities,
        &Friendly,
        Option<&mut ProjectileContacted>,
    )>,
    q_friendly: Query<&Friendly>,
    q_no_dispose_projectile: Query<(), With<projectile::NoDisposeProjectile>>,
) {
    q_contact_damage.par_iter_mut().for_each(
        |(contact_damage, colliding_entities, friendly, mut contacted)| {
            for entity in colliding_entities.iter() {
                if q_no_dispose_projectile.get(*entity).is_err()
                    && contacted
                        .as_mut()
                        .is_none_or(|contacted| contacted.insert(*entity))
                    && q_friendly
                        .get(*entity)
                        .is_ok_and(|target_friendly| target_friendly.0 ^ friendly.0)
                {
                    commands.command_scope(|mut commands| {
                        commands.trigger(Damage {
                            entity: *entity,
                            order: contact_damage.order,
                            value: contact_damage.value,
                        });
                    })
                }
            }
        },
    );
}
