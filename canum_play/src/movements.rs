use crate::health::Shields;
use crate::prelude::*;

pub(super) struct MovementsPlugin;
impl Plugin for MovementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, do_speed_shrink);
        app.add_systems(FixedPreUpdate, refresh_dash_timers);
        app.add_systems(FixedUpdate, perform_dash);
        app.add_observer(start_dash);
        app.add_systems(
            FixedLast,
            (update_forced_velocity, update_velocity, speed_decay).chain(),
        );
        app.add_systems(FixedLast, auto_flip);
    }
}

/// The entity shrinks their animation and hitbox by a base number when their speed increases.
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
        shields.insert(crate::consts::order::DASH_INVINC, i32::MAX);
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
                || timers.total.is_finished()
                    && shields.contains_key(&crate::consts::order::DASH_INVINC)
            {
                shields.remove(&crate::consts::order::DASH_INVINC);
            }
        });
}

/// Marks this entity to decrease speed gradually, for the part minus forced velocity.
#[derive(Component, Debug, Clone)]
#[require(ForcedVelocity)]
pub struct SpeedDecay(pub f32);
impl Default for SpeedDecay {
    fn default() -> Self {
        Self(0.5)
    }
}

/// Makes up part of the actual `LinearVelocity`, and this part is unaffected by `SpeedDecay`.
#[derive(Component, Debug, Clone, Default, Deref, DerefMut)]
#[require(LinearVelocity, PrevForcedVelocity)]
pub struct ForcedVelocity(pub Vec2);

/// A child of `ForcedVelocity`. Individual components can modify their part of velocity.
///
/// If there are no partial velocities, `ForcedVelocity` is directly modified.
#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct PartialVelocity {
    #[deref]
    pub velocity: Vec2,
    pub linked: Option<Entity>,
}
impl PartialVelocity {
    /// Creates a `PartialVelocity` with a linked entity. When the linked entity despawns, the velocity will despawn itself.
    pub fn linked(entity: Entity) -> Self {
        Self {
            velocity: Vec2::ZERO,
            linked: Some(entity),
        }
    }

    /// Creates a `PartialVelocity` without a linked entity. It will not despawn itself.
    pub fn unlinked() -> Self {
        Self {
            velocity: Vec2::ZERO,
            linked: None,
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
struct PrevForcedVelocity(Vec2);

fn update_forced_velocity(
    commands: ParallelCommands,
    mut q_forced: Query<(&mut ForcedVelocity, &Children)>,
    q_partial: Query<Ref<PartialVelocity>>,
    q_ok: Query<()>,
) {
    q_forced.par_iter_mut().for_each(|(mut forced, children)| {
        let mut total = Vec2::ZERO;
        let mut any_changed = false;
        for child in children.iter() {
            let Ok(partial) = q_partial.get(child) else {
                continue;
            };
            if let Some(linked) = partial.linked
                && q_ok.get(linked).is_err()
            {
                commands.command_scope(|mut commands| {
                    commands.entity(child).despawn();
                });
                any_changed = true;
                continue;
            }
            total += partial.velocity;
            if partial.is_changed() {
                any_changed = true;
            }
        }
        if any_changed {
            forced.0 = total;
        }
    });
}

fn update_velocity(
    mut q_forced: Query<(
        Ref<ForcedVelocity>,
        &mut LinearVelocity,
        &mut PrevForcedVelocity,
    )>,
) {
    q_forced
        .par_iter_mut()
        .for_each(|(forced, mut linear_velocity, mut prev)| {
            if forced.is_changed() {
                linear_velocity.0 += forced.0 - prev.0;
                prev.0 = forced.0;
            }
        });
}
fn speed_decay(mut q_velocity: Query<(&SpeedDecay, &mut LinearVelocity, &ForcedVelocity)>) {
    q_velocity
        .par_iter_mut()
        .for_each(|(decay, mut linear_velocity, forced)| {
            let amount = (linear_velocity.0 - forced.0) * decay.0;
            linear_velocity.0 -= amount;
        });
}

/// Automatically flips the sprite's selected axis based on velocity.
/// You must give the sprite's default direction by sign (1 or -1).
#[derive(Component, Debug, Clone, Default)]
#[require(Animation)]
pub struct AutoFlip {
    x: i8,
    y: i8,
}
impl AutoFlip {
    pub const FLIP_RIGHT: Self = Self::flip_x(1);
    pub const FLIP_LEFT: Self = Self::flip_x(-1);
    pub const FLIP_UP: Self = Self::flip_y(1);
    pub const FLIP_DOWN: Self = Self::flip_y(-1);

    pub const fn flip_x(x: i8) -> Self {
        Self { x, y: 0 }
    }
    pub const fn flip_y(y: i8) -> Self {
        Self { x: 0, y }
    }
}

fn auto_flip(mut q_flip: Query<(&AutoFlip, &LinearVelocity, &mut Sprite)>) {
    q_flip
        .par_iter_mut()
        .for_each(|(auto_flip, linear_velocity, mut sprite)| {
            if auto_flip.x != 0 && linear_velocity.x.abs() >= 10.0 {
                let should_flip = (linear_velocity.x > 0.0) ^ (auto_flip.x > 0);
                if sprite.flip_x != should_flip {
                    sprite.flip_x = should_flip;
                }
            }
            if auto_flip.y != 0 && linear_velocity.y.abs() >= 10.0 {
                let should_flip = (linear_velocity.y > 0.0) ^ (auto_flip.y > 0);
                if sprite.flip_y != should_flip {
                    sprite.flip_y = should_flip;
                }
            }
        });
}
