use super::*;
use crate::prelude::*;

pub(super) struct PlayerAttackPlugin;

impl Plugin for PlayerAttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, (init_weapons, init_filed));
    }
}

/// Calls the player to use attack. How to respond depends on weapon itself.
#[derive(EntityEvent, Debug)]
pub struct Attack {
    pub entity: Entity,
}

/// The weapons that the player chooses.
#[derive(Component, Debug, Deref, DerefMut, Default)]
pub struct Weapons(pub Vec<Option<Entity>>);

fn init_weapons(mut q_weapons: Query<&mut Weapons>, save: Res<Save>) {
    for mut weapons in q_weapons.iter_mut() {
        if weapons.is_added() {
            weapons.resize(save.progress.weapon_slots, None);
        }
    }
}

/// Marks a player-spawn projectile.
#[derive(Component)]
#[require(
    crate::health::Friendly(true),
    Animation,
    RigidBody::Kinematic,
    Collider,
    crate::health::ContactDamage
)]
pub struct PlayerProjectile;

#[derive(Component, Debug)]
#[require(FiledTimer, Transform)]
pub struct Filed {
    pub interval: f32,
    pub damage: i32,
    pub order: u8,
    pub size: Vec2,
    pub speed: f32,
}
#[derive(Component, Default)]
struct FiledTimer {
    interval: Timer,
}

impl Default for Filed {
    fn default() -> Self {
        Self {
            interval: 0.1,
            damage: 10,
            order: 50,
            size: Vec2::new(5.0, 10.0),
            speed: 450.0,
        }
    }
}

fn init_filed(
    mut commands: Commands,
    mut q_filed: Query<(Entity, &mut FiledTimer, &Filed), Added<Filed>>,
) {
    for (entity, mut filed_timer, filed) in q_filed.iter_mut() {
        filed_timer.interval = Timer::from_seconds(filed.interval, TimerMode::Repeating);
        commands.entity(entity).observe(filed_shoot);
    }
}
fn filed_shoot(
    event: On<Attack>,
    mut commands: Commands,
    mut q_filed: Query<(&Filed, &mut FiledTimer, &GlobalTransform, &ChildOf)>,
    q_player: Query<&PlayerShoot>,
    time: Res<Time>,
) {
    let Ok((filed, mut filed_timer, global_transform, parent)) = q_filed.get_mut(event.entity)
    else {
        return;
    };
    let Ok(player_shoot) = q_player.get(parent.0) else {
        return;
    };
    filed_timer.interval.tick(time.delta());
    if filed_timer.interval.just_finished() {
        let direction = Vec2::from_angle(player_shoot.0);
        let position = global_transform.translation().xy() + direction * filed.size.y;
        let transform = Transform::from_translation(Vec3::new(position.x, position.y, 1.0))
            .with_rotation(Quat::from_rotation_z(
                player_shoot.0 - std::f32::consts::FRAC_PI_2,
            ));
        commands.spawn((
            PlayerProjectile,
            transform,
            Animation::new("Filed", filed.size),
            Collider::capsule(filed.size.x, filed.size.y),
            LinearVelocity(direction * filed.speed),
        ));
    }
}
