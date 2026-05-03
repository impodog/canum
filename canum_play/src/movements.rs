use crate::health::Shields;
use crate::prelude::*;

pub(super) struct MovementsPlugin;
impl Plugin for MovementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, do_speed_shrink);
        app.add_systems(FixedPreUpdate, refresh_dash_timers);
        app.add_systems(FixedUpdate, (perform_dash, update_dash_velocity).chain());
        app.add_observer(start_dash);
        app.world_mut().register_component_hooks::<Dash>().on_add(
            |mut world, HookContext { entity, .. }| {
                world.commands().spawn((ChildOf(entity), DashVelocity));
            },
        );
        app.add_systems(
            FixedLast,
            (
                update_forced_velocity,
                update_velocity,
                (speed_decay, update_direct_velocity),
            )
                .chain(),
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

/// Add this part to `PartialVelocity`, so that it won't be included in speed shrink calculation.
#[derive(Component, Default)]
pub struct SpeedShrinkExclude;

fn do_speed_shrink(
    mut q_entity: Query<(
        &SpeedShrink,
        &mut Collider,
        &LinearVelocity,
        &mut Animation,
        &Children,
    )>,
    q_exclude: Query<&PartialVelocity, With<SpeedShrinkExclude>>,
) {
    q_entity.par_iter_mut().for_each(
        |(shrink, mut collider, linear_velocity, mut animation, children)| {
            let mut velocity = **linear_velocity;
            for child in children.iter() {
                if let Ok(partial_velocity) = q_exclude.get(child) {
                    velocity -= **partial_velocity;
                }
            }
            let factor = (0.5f32).powf(velocity.length() / shrink.0);
            collider.set_scale(Vec2::new(factor, 1.0), 6);
            animation.scale = Vec2::new(factor, 1.0);
        },
    );
}

#[derive(Component, Debug, Clone)]
#[require(ForcedVelocity, DashTimers, Shields)]
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
            max_speed: 400.0,
            total_duration: 0.20,
            invincible_duration: 0.02,
            cooldown_duration: 0.6,
            sound: "Dash".to_owned(),
        }
    }
}

/// Marks a child containing partial velocity controlled by `Dash`.
#[derive(Component, Default)]
#[require(PartialVelocity::unlinked())]
struct DashVelocity;

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
fn perform_dash(mut q_dash: Query<(&Dash, &mut DashTimers, &mut Shields)>, time: Res<Time>) {
    q_dash
        .par_iter_mut()
        .for_each(|(_dash, mut timers, mut shields)| {
            if !timers.total.is_finished() {
                timers.total.tick(time.delta());
                timers.invincible.tick(time.delta());
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
fn update_dash_velocity(
    mut q_velocity: Query<(&mut PartialVelocity, &ChildOf), With<DashVelocity>>,
    q_timer: Query<&DashTimers>,
) {
    q_velocity
        .par_iter_mut()
        .for_each(|(mut partial_velocity, parent)| {
            let Ok(timers) = q_timer.get(parent.0) else {
                return;
            };
            if !timers.total.is_finished() {
                let diff = timers.target_velocity - **partial_velocity;
                let length = diff.length();
                if let Some(diff) = diff.try_normalize() {
                    **partial_velocity += diff * (100.0f32).min(length);
                }
            } else {
                let subtract =
                    partial_velocity.normalize_or_zero() * (25.0f32).min(partial_velocity.length());
                **partial_velocity -= subtract;
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

/// If an entity has no rigid body, it can use this marker to move directly using `LinearVelocity`.
/// If the entity has a rigid body this does nothing.
#[derive(Component, Default)]
#[require(LinearVelocity)]
pub struct DirectVelocity;

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
    ///
    /// This is useful for external entities that forces others to move.
    pub fn linked(entity: Entity) -> Self {
        Self {
            velocity: Vec2::ZERO,
            linked: Some(entity),
        }
    }

    /// Creates a `PartialVelocity` without a linked entity. It will not despawn itself.
    ///
    /// This is useful for child entities that describe movement itself.
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

#[allow(clippy::type_complexity)]
fn update_direct_velocity(
    mut q_velocity: Query<
        (&mut Transform, &LinearVelocity),
        (With<DirectVelocity>, Without<RigidBody>),
    >,
    time: Res<Time>,
) {
    q_velocity
        .par_iter_mut()
        .for_each(|(mut transform, linear_velocity)| {
            transform.translation.x += linear_velocity.x * time.delta_secs();
            transform.translation.y += linear_velocity.y * time.delta_secs();
        });
}

/// Automatically flips the sprite's selected axis based on velocity.
/// You must give the sprite's default direction by sign (1 or -1).
#[derive(Component, Debug, Clone, Default)]
#[require(Animation, AutoFlipStatus)]
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

/// Stores the absolute axis direction (1.0 or -1.0) that auto flip has affected.
/// This is used for coordination with animation.
#[derive(Component, Debug, Default)]
pub struct AutoFlipStatus {
    pub x: f32,
    pub y: f32,
}

fn auto_flip(mut q_flip: Query<(&AutoFlip, &ForcedVelocity, &mut Sprite, &mut AutoFlipStatus)>) {
    q_flip
        .par_iter_mut()
        .for_each(|(auto_flip, forced_velocity, mut sprite, mut status)| {
            if auto_flip.x != 0 && forced_velocity.x.abs() >= 10.0 {
                if status.x == 0.0 {
                    status.x = 1.0;
                }
                let should_flip = (forced_velocity.x > 0.0) ^ (auto_flip.x > 0);
                if sprite.flip_x != should_flip {
                    sprite.flip_x = should_flip;
                    status.x = if forced_velocity.x > 0.0 { 1.0 } else { -1.0 };
                }
            }
            if auto_flip.y != 0 && forced_velocity.y.abs() >= 10.0 {
                if status.y == 0.0 {
                    status.y = 1.0;
                }
                let should_flip = (forced_velocity.y > 0.0) ^ (auto_flip.y > 0);
                if sprite.flip_y != should_flip {
                    sprite.flip_y = should_flip;
                    status.x = if forced_velocity.y > 0.0 { 1.0 } else { -1.0 };
                }
            }
        });
}
