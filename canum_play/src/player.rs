use crate::prelude::*;

pub(super) struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(respond_player_move);
        app.add_systems(FixedPreUpdate, init_player_acc);
        app.add_systems(FixedPostUpdate, decay_player_acc);
    }
}

#[derive(Component, Default)]
#[require(
    Animation,
    RigidBody::Kinematic,
    Collider::circle(16.0),
    PlayerShoot,
    PlayerAcc,
    crate::general::SpeedShrink(400.0)
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
    pub angular: f32,
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
        &mut LinearVelocity,
        &mut AngularVelocity,
        &Rotation,
    )>,
) {
    for (mut acc, mut linear_velocity, mut angular_velocity, rotation) in q_player.iter_mut() {
        if !acc.changed {
            acc.linear = (acc.linear - 0.05).max(0.0);
            acc.angular = (acc.angular - 0.1).max(0.0);
            if rotation.cos < 0.99 {
                angular_velocity.0 = -rotation.as_radians().signum() * (1.01 - rotation.cos) * 10.0;
            }
            if linear_velocity.length_squared() > 0.01 {
                linear_velocity.0 *= 0.8;
            }
        }
    }
}

fn respond_player_move(
    event: On<PlayerMove>,
    mut q_player: Query<(
        &mut PlayerAcc,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &Transform,
    )>,
) -> Result<()> {
    fn wrap_angle(angle: f32) -> f32 {
        if angle > std::f32::consts::PI {
            angle - std::f32::consts::PI * 2.0
        } else if angle <= -std::f32::consts::PI {
            angle + std::f32::consts::PI * 2.0
        } else {
            angle
        }
    }

    let (mut acc, mut linear_velocity, mut angular_velocity, transform) =
        q_player.get_mut(event.entity)?;
    acc.changed = true;
    acc.linear += (1.0 - acc.linear) * 0.333;
    acc.angular += (1.0 - acc.angular) * 0.666;
    {
        let target_velocity =
            Vec2::new(event.rot.cos() * event.mult, event.rot.sin() * event.mult) * 150.0;
        let diff = target_velocity - linear_velocity.0;
        let diff_len = diff.length();
        if let Some(diff) = diff.try_normalize() {
            linear_velocity.0 += diff * acc.linear * (10.0f32).min(diff_len);
        }
    }
    {
        let current_rotation =
            wrap_angle(transform.rotation.to_euler(EulerRot::ZYX).0 + std::f32::consts::FRAC_PI_2);
        let diff = wrap_angle(event.rot - current_rotation);
        if diff.abs() > 1e-2 {
            if (0.0..std::f32::consts::PI).contains(&diff) {
                angular_velocity.0 = acc.angular * 10.0;
            } else {
                angular_velocity.0 = -acc.angular * 10.0;
            }
        } else {
            angular_velocity.0 = 0.0;
        }
    }
    Ok(())
}
