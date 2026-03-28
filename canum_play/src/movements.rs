use crate::health::Shields;
use crate::prelude::*;

pub(super) struct MovementsPlugin;
impl Plugin for MovementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, do_speed_shrink);
        app.add_systems(FixedPreUpdate, refresh_dash_timers);
        app.add_systems(FixedUpdate, perform_dash);
        app.add_systems(FixedPostUpdate, remove_out_of_bound_projectiles);
        app.add_observer(start_dash);
    }
}

/// The entity shrinks their animation and hitbox and hitbox by a base number when their speed increases.
///
/// For each time the velocity of given number, the size is reduced by half.
#[derive(Component, Debug, Clone)]
#[require(Animation)]
pub struct SpeedShrink(pub f32);

fn do_speed_shrink(
    mut q_entity: Query<(&SpeedShrink, &mut Collider, &LinearVelocity, &mut Animation)>,
) {
    q_entity
        .par_iter_mut()
        .for_each(|(shrink, mut collider, linear_velocity, mut animation)| {
            let factor = (0.5f32).powf(linear_velocity.length() / shrink.0);
            collider.set_scale(Vec2::new(factor, 1.0), 6);
            animation.scale = Vec2::new(factor, 1.0);
        });
}

#[derive(Component, Debug, Clone)]
#[require(LinearVelocity, DashTimers, Shields)]
pub struct Dash {
    pub max_speed: f32,
    pub total_duration: f32,
    pub invincible_duration: f32,
    pub cooldown_duration: f32,
    pub sound: String,
}
#[derive(Component, Debug, Clone, Default)]
pub struct DashTimers {
    pub total: Timer,
    pub cooldown: Timer,
    pub invincible: Timer,
    pub target_velocity: Vec2,
}

impl Dash {
    pub const SHIELD_ORDER: u8 = 230;
}
impl Default for Dash {
    fn default() -> Self {
        Self {
            max_speed: 450.0,
            total_duration: 0.20,
            invincible_duration: 0.04,
            cooldown_duration: 0.4,
            sound: "Dash".to_owned(),
        }
    }
}

fn refresh_dash_timers_with(dash: &Dash, timers: &mut DashTimers) {
    timers.total = Timer::from_seconds(dash.total_duration, TimerMode::Once);
    timers.invincible = Timer::from_seconds(dash.invincible_duration, TimerMode::Once);
    timers.cooldown = Timer::from_seconds(dash.cooldown_duration, TimerMode::Once);
}
fn refresh_dash_timers(mut q_modified: Query<(&Dash, &mut DashTimers), Changed<Dash>>) {
    q_modified.par_iter_mut().for_each(|(dash, mut timers)| {
        refresh_dash_timers_with(dash, timers.as_mut());
        timers.total.finish();
        timers.invincible.finish();
        timers.cooldown.finish();
    });
}

#[derive(EntityEvent, Debug)]
pub struct StartDash {
    pub entity: Entity,
    /// This is later multiplied by `Dash::max_speed`.
    pub base_velocity: Vec2,
}

fn start_dash(
    event: On<StartDash>,
    mut q_dash: Query<(&Dash, &mut DashTimers, &mut Shields)>,
    mut commands: Commands,
) {
    let Ok((dash, mut timers, mut shields)) = q_dash.get_mut(event.entity) else {
        return;
    };
    if !timers.cooldown.is_finished() {
        return;
    }
    commands.spawn(canum_res::sound::Sound::new(&dash.sound));
    refresh_dash_timers_with(dash, timers.as_mut());
    if dash.invincible_duration > 0.0 {
        shields.insert(Dash::SHIELD_ORDER, i32::MAX);
    }
    timers.target_velocity = event.base_velocity * dash.max_speed;
}
fn perform_dash(
    mut q_dash: Query<(&Dash, &mut DashTimers, &mut LinearVelocity, &mut Shields)>,
    time: Res<Time>,
) {
    q_dash
        .par_iter_mut()
        .for_each(|(_dash, mut timers, mut linear_velocity, mut shields)| {
            if !timers.total.is_finished() {
                timers.total.tick(time.delta());
                timers.invincible.tick(time.delta());
                let diff = timers.target_velocity - linear_velocity.0;
                let length = diff.length();
                if let Some(diff) = diff.try_normalize() {
                    linear_velocity.0 += diff * (100.0f32).min(length);
                }
            } else if !timers.cooldown.is_finished() {
                timers.cooldown.tick(time.delta());
            }
            if timers.invincible.is_finished()
                || timers.total.is_finished() && shields.contains_key(&Dash::SHIELD_ORDER)
            {
                shields.remove(&Dash::SHIELD_ORDER);
            }
        });
}

/// Marks a projectile either by player or enemy.
#[derive(Component, Debug, Default)]
#[require(
    crate::SessionOnly,
    Transform,
    Collider,
    RigidBody::Kinematic,
    crate::health::Friendly
)]
pub struct Projectile;

fn remove_out_of_bound_projectiles(
    commands: ParallelCommands,
    q_projectile: Query<(Entity, &Transform), With<Projectile>>,
) {
    let virtual_size = (
        CONFIG.display.virtual_size.0 as f32,
        CONFIG.display.virtual_size.1 as f32,
    );
    q_projectile.par_iter().for_each(|(entity, transform)| {
        if transform.translation.x.abs() > virtual_size.0
            || transform.translation.y.abs() > virtual_size.1
        {
            commands.command_scope(|mut commands| {
                commands.entity(entity).despawn();
            });
        }
    });
}
