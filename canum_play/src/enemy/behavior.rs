use bevy::math::FloatPow;
use std::{collections::BTreeSet, time::Duration};

use crate::prelude::*;

pub(super) struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerAveragePosition>();
        app.add_systems(
            FixedPreUpdate,
            (init_behavior_weights, update_player_average_position),
        );
        app.add_systems(FixedUpdate, (base_by_distance, base_farther_better));
        app.add_systems(FixedPostUpdate, start_behavior);
        app.add_observer(end_behavior);
    }
}

/// Event to call an behavior to do its part.
#[derive(EntityEvent, Debug, Clone)]
pub struct BehaveStart {
    pub entity: Entity,
    /// The target entity(boss) that trigger this behavior.
    pub target: Entity,
}
/// The behavior must send this back to end itself.
#[derive(EntityEvent, Debug, Clone)]
pub struct BehaveEnd {
    pub entity: Entity,
    /// The enemy no longer triggers behavior in a cooldown duration.
    pub cooldown: Duration,
}

/// The parent entity of all behaviors.
#[derive(Component, Debug, Clone, Default)]
#[require(BehaviorManagerInfo, Transform)]
pub struct BehaviorManager {
    /// The target that all behaviors affect. Defaults to the parent of the manager.
    pub target: Option<Entity>,
}
impl BehaviorManager {
    /// Manages the parent entity, with activity settings.
    pub fn new() -> Self {
        Self { target: None }
    }
}

#[derive(Component, Default, Debug)]
struct BehaviorManagerInfo {
    occupied: BTreeSet<String>,
    cooldown: Timer,
}

fn start_behavior(
    commands: ParallelCommands,
    mut q_manager: Query<(
        &BehaviorManager,
        &mut BehaviorManagerInfo,
        &Children,
        &ChildOf,
    )>,
    q_behavior: Query<(&Behavior, &Weight)>,
    time: Res<Time>,
) {
    use rand::seq::IndexedRandom;

    q_manager
        .par_iter_mut()
        .for_each(|(manager, mut info, children, parent)| {
            info.cooldown.tick(time.delta());
            if !info.cooldown.is_finished() {
                return;
            }
            info.cooldown = Default::default();
            let mut behaviors = Vec::new();
            for child in children.iter() {
                let Ok((behavior, weight)) = q_behavior.get(child) else {
                    continue;
                };
                // Filter behaviors whose required resources are occupied.
                if behavior
                    .occupies
                    .intersection(&info.occupied)
                    .next()
                    .is_some()
                {
                    continue;
                }

                let weight = weight.calc();
                if weight > 1e-6 {
                    behaviors.push((weight, child));
                }
            }
            let Ok((_, entity)) =
                behaviors.choose_weighted(&mut rand::rng(), |(weight, _)| *weight)
            else {
                return;
            };
            let Ok((behavior, _)) = q_behavior.get(*entity) else {
                return;
            };
            info.occupied.extend(behavior.occupies.iter().cloned());
            if let Some(cooldown) = behavior.cooldown {
                info.cooldown = Timer::new(cooldown, TimerMode::Once);
            }
            info!("Triggering behavior {}", behavior.name);
            commands.command_scope(|mut commands| {
                commands.trigger(BehaveStart {
                    entity: *entity,
                    target: manager.target.unwrap_or(parent.0),
                });
            });
        });
}

fn end_behavior(
    event: On<BehaveEnd>,
    q_behavior: Query<(&Behavior, &ChildOf)>,
    mut q_manager: Query<&mut BehaviorManagerInfo>,
) {
    let Ok((behavior, parent)) = q_behavior.get(event.entity) else {
        warn!(
            "Behavior {} without a parent, this will not be automatically triggered.",
            event.entity
        );
        return;
    };
    let Ok(mut info) = q_manager.get_mut(parent.0) else {
        warn!(
            "Parent {} of behavior {} is not a manager.",
            parent.0, event.entity
        );
        return;
    };
    let new_duration = info.cooldown.duration() + event.cooldown;
    info.cooldown.set_duration(new_duration);
    for resource in behavior.occupies.iter() {
        info.occupied.remove(resource);
    }
}

/// A single boss behavior, describing effects while it is between start and end.
#[derive(Component, Debug, Clone, Default)]
#[require(Weight, Transform)]
pub struct Behavior {
    pub name: String,
    /// This behavior occupies these resources, and they must be vacant for this behavior to trigger.
    pub occupies: BTreeSet<String>,
    /// The weight of this behavior, when no modifiers are applied.
    pub default_weight: f32,
    /// The cooldown, immediately after behavior starts, before any chance for next behavior.
    pub cooldown: Option<Duration>,
}
impl Behavior {
    pub fn new<T>(
        name: impl Into<String>,
        default_weight: f32,
        occupies: impl IntoIterator<Item = T>,
    ) -> Self
    where
        T: Into<String>,
    {
        Self {
            name: name.into(),
            occupies: occupies.into_iter().map(|item| item.into()).collect(),
            default_weight,
            cooldown: None,
        }
    }

    pub fn new_with_cooldown<T>(
        name: impl Into<String>,
        default_weight: f32,
        occupies: impl IntoIterator<Item = T>,
        cooldown: Duration,
    ) -> Self
    where
        T: Into<String>,
    {
        Self::new(name, default_weight, occupies).with_cooldown(cooldown)
    }

    pub fn with_cooldown(mut self, cooldown: Duration) -> Self {
        self.cooldown = Some(cooldown);
        self
    }
}

/// The behavior triggers with this weight. This is re-calculated each frame.
#[derive(Component, Debug, Clone, Default)]
pub struct Weight {
    pub predetermined: f32,
    pub base: f32,
    pub multiplier: f32,
}
impl Weight {
    pub fn calc(&self) -> f32 {
        self.predetermined * self.base * self.multiplier
    }
}

fn init_behavior_weights(mut q_behavior: Query<(&Behavior, &mut Weight)>) {
    q_behavior
        .par_iter_mut()
        .for_each(|(behavior, mut weight)| {
            weight.predetermined = behavior.default_weight;
            weight.base = 1.0;
            weight.multiplier = 1.0;
        });
}

#[derive(Resource, Debug, Default, Deref, DerefMut)]
pub struct PlayerAveragePosition(pub Vec2);
fn update_player_average_position(
    mut position: ResMut<PlayerAveragePosition>,
    q_player: Query<&GlobalTransform, With<crate::player::Player>>,
) {
    let mut center = Vec2::ZERO;
    let mut count = 0;
    for transform in q_player.iter() {
        center += transform.translation().xy();
        count += 1;
    }
    if count > 0 {
        center /= count as f32;
        position.0 = center;
    }
}

/// Decides weight base by distance to the player.
/// This uses gaussian probability.
#[derive(Component, Debug, Default)]
#[require(Transform)]
pub struct BaseByDistance {
    pub mean: f32,
    pub deviation: f32,
}
impl BaseByDistance {
    pub fn new(mean: f32, deviation: f32) -> Self {
        Self { mean, deviation }
    }
}
fn base_by_distance(
    mut q_behavior: Query<(&mut Weight, &BaseByDistance, &GlobalTransform)>,
    average_position: Res<PlayerAveragePosition>,
) {
    q_behavior
        .par_iter_mut()
        .for_each(|(mut weight, modifier, global_transform)| {
            let distance = global_transform
                .translation()
                .xy()
                .distance(**average_position);
            weight.base = std::f32::consts::E
                .powf(-(distance - modifier.mean).squared() * 0.5 / modifier.deviation.squared());
        });
}

/// The farther the player is, the more likely this will player.
/// This uses inverse function.
#[derive(Component, Debug, Default)]
#[require(Transform)]
pub struct BaseFartherBetter {
    /// The probability increases from 0.0, after distance > `start`.
    pub start: f32,
    pub unit_length: f32,
}
impl BaseFartherBetter {
    pub fn new(start: f32, unit_length: f32) -> Self {
        Self { start, unit_length }
    }
}
fn base_farther_better(
    mut q_behavior: Query<(&mut Weight, &BaseFartherBetter, &GlobalTransform)>,
    average_position: Res<PlayerAveragePosition>,
) {
    q_behavior
        .par_iter_mut()
        .for_each(|(mut weight, modifier, global_transform)| {
            let distance = global_transform
                .translation()
                .xy()
                .distance(**average_position);
            if distance >= modifier.start {
                weight.base =
                    1.0 - 1.0 / ((distance - modifier.start) / modifier.unit_length + 1.0);
            } else {
                weight.base = 0.0;
            }
        });
}
