pub mod attack;

use crate::prelude::*;

pub(super) struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(respond_player_move);
        app.add_systems(FixedPreUpdate, init_player_acc);
        app.add_systems(FixedPostUpdate, decay_player_acc);
        app.add_plugins(attack::PlayerAttackPlugin);
    }
}

#[derive(Component, Default)]
#[require(
    Animation,
    Transform::from_translation(Vec3::new(0.0, 0.0, 24.37)),
    RigidBody::Dynamic,
    Collider::circle(10.0),
    Mass(10.0),
    CollisionEventsEnabled,
    PlayerShoot,
    PlayerAcc,
    attack::Weapons,
    crate::movements::SpeedShrink(800.0),
    crate::movements::Dash,
    crate::health::Friendly(true)
)]
pub struct Player;

/// Stores which player can be directly controled.
#[derive(Resource, Debug, Deref, DerefMut)]
pub struct PrimaryPlayer(pub Entity);

/// Stores the shooting direction the player is facing.
#[derive(Component, Default, Debug)]
pub struct PlayerShoot(pub f32);

/// Tweaks the player control experience with acceleration.
#[derive(Component, Default, Debug)]
pub struct PlayerAcc {
    pub linear: f32,
    pub changed: bool,
}

/// Instructs the player to move in a direction, with a multiplier.
/// The actual velocity is affected friction and acceleration.
#[derive(EntityEvent, Debug)]
pub struct PlayerMove {
    pub entity: Entity,
    pub rot: f32,
    pub mult: f32,
}

/// Resets `changed` flag.
fn init_player_acc(mut q_player: Query<&mut PlayerAcc>) {
    for mut acc in q_player.iter_mut() {
        acc.changed = false;
    }
}
/// If acc is not changed, acc and velocity decays over time.
fn decay_player_acc(
    mut q_player: Query<(
        &mut PlayerAcc,
        &mut PlayerShoot,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &mut crate::movements::DashTimers,
        &Rotation,
    )>,
) {
    fn wrap_angle(angle: f32) -> f32 {
        if angle > std::f32::consts::PI {
            angle - std::f32::consts::PI * 2.0
        } else if angle <= -std::f32::consts::PI {
            angle + std::f32::consts::PI * 2.0
        } else {
            angle
        }
    }

    for (
        mut acc,
        mut player_shoot,
        mut linear_velocity,
        mut angular_velocity,
        mut dash_timers,
        rotation,
    ) in q_player.iter_mut()
    {
        if !acc.changed {
            dash_timers.total.finish();
            acc.linear = (acc.linear - 0.05).max(0.0);
            if rotation.cos < 0.99 {
                angular_velocity.0 = -rotation.as_radians().signum() * (1.01 - rotation.cos) * 10.0;
            }
            if linear_velocity.length_squared() > 0.01 {
                linear_velocity.0 *= 0.8;
            }
        }
        {
            let target_rotation = linear_velocity.to_angle();
            let current_rotation = wrap_angle(rotation.as_radians() + std::f32::consts::FRAC_PI_2);
            let diff = wrap_angle(target_rotation - current_rotation);
            if diff.abs() > 1e-2 {
                let base = linear_velocity.length() / 100.0
                    * (15.0f32)
                    * (diff.abs() / std::f32::consts::PI + 0.05);
                if (0.0..std::f32::consts::PI).contains(&diff) {
                    angular_velocity.0 = base;
                } else {
                    angular_velocity.0 = -base;
                }
            } else {
                angular_velocity.0 = 0.0;
            }
            player_shoot.0 = current_rotation;
        }
    }
}

fn respond_player_move(
    event: On<PlayerMove>,
    mut q_player: Query<(
        &mut PlayerAcc,
        &mut LinearVelocity,
        &crate::movements::DashTimers,
    )>,
) -> Result<()> {
    const PLAYER_SPEED: f32 = 200.0;

    let (mut acc, mut linear_velocity, dash_timers) = q_player.get_mut(event.entity)?;
    acc.changed = true;
    acc.linear += (1.0 - acc.linear) * 0.333;
    {
        let original_speed = linear_velocity.length();
        let target_speed = if dash_timers.total.is_finished() {
            PLAYER_SPEED
        } else {
            PLAYER_SPEED.max(original_speed)
        };
        let target_velocity =
            Vec2::new(event.rot.cos(), event.rot.sin()) * event.mult * target_speed;
        let diff = target_velocity - linear_velocity.0;
        let diff_len = diff.length();
        if let Some(diff) = diff.try_normalize() {
            linear_velocity.0 += diff * acc.linear * (10.0f32).min(diff_len);
        }
    }
    Ok(())
}
