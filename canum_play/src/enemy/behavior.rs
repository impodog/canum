use bevy::math::FloatPow;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    f32,
    time::Duration,
};

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum BehaviorProcess {
    Init,
    Calc,
    Trigger,
}

pub(super) struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerAveragePosition>();
        app.configure_sets(
            FixedPreUpdate,
            (
                BehaviorProcess::Init,
                BehaviorProcess::Calc,
                BehaviorProcess::Trigger,
            ),
        );
        app.add_systems(
            FixedPreUpdate,
            (init_behavior_weights, update_player_average_position).in_set(BehaviorProcess::Init),
        );
        app.add_systems(
            FixedPreUpdate,
            (
                base_by_distance,
                base_farther_better,
                multiplier_manual,
                multiplier_by_speed,
                multiplier_when_resource_occupied,
            )
                .in_set(BehaviorProcess::Calc),
        );
        app.add_systems(
            FixedPreUpdate,
            select_behavior.in_set(BehaviorProcess::Trigger),
        );

        app.add_observer(start_behavior)
            .add_observer(end_behavior)
            .add_observer(intervene_behavior);
        app.world_mut()
            .register_component_hooks::<BehaviorManager>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(intervene_behavior_manager);
            });
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
    /// Locks all behaviors in this duration.
    pub cooldown: Duration,
    /// The enemy no longer triggers behavior that takes this resource in a cooldown duration.
    pub occupies: Vec<(String, Duration)>,
}
/// Event to force a behavior to end before the next frame. This is used for staggering and such.
///
/// This auto propagates in Use 1, but not in Use 2.
///
/// ### Use 1
///
/// For children behaviors, you need to implement intervention event response logic for each behavior, if necessary.
///
/// ### Use 2
///
/// External code may call this on the behavior manager to intervene all running behaviors. This
/// also disables the behavior manager. When used like this, `target` is unnecessary.
#[derive(EntityEvent, Debug, Clone)]
#[entity_event(auto_propagate)]
pub struct BehaveIntervene {
    pub entity: Entity,
    /// The target entity(boss) that trigger this behavior.
    pub target: Entity,
}

impl BehaveIntervene {
    pub fn new_for_manager(entity: Entity) -> Self {
        Self {
            entity,
            // This is unnecessary
            target: entity,
        }
    }
}

/// Utility for creating `BehaveEnd::occupies`. This handles empty vectors correctly.
#[macro_export]
macro_rules! occupies {
    [] => {
        vec![]
    };
    [$($item: expr),+] => {
        $crate::enemy::behavior::occupies_helper([$($item),+])
    }
}

/// Utility function for filling the `BehaveEnd::occupies` field.
pub fn occupies_helper<S>(iter: impl IntoIterator<Item = (S, f32)>) -> Vec<(String, Duration)>
where
    S: Into<String>,
{
    iter.into_iter()
        .map(|(resource, seconds)| (resource.into(), Duration::from_secs_f32(seconds)))
        .collect()
}

/// The parent entity of all behaviors.
#[derive(Component, Debug, Clone)]
#[require(BehaviorManagerInfo, Transform)]
pub struct BehaviorManager {
    /// The target that all behaviors affect. Defaults to the parent of the manager. If parent does not exist, defaults to itself.
    pub target: Option<Entity>,
    /// Temporarily disables the behavior manager for a manual behavior.
    pub disabled: bool,
}
impl BehaviorManager {
    /// Manages the parent entity, with activity settings.
    pub fn new() -> Self {
        Self {
            target: None,
            disabled: false,
        }
    }
}
impl Default for BehaviorManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Stores runtime information of the manager. External code may only access read-only.
#[derive(Component, Debug)]
pub struct BehaviorManagerInfo {
    occupied: BTreeMap<String, Timer>,
    /// All running behaviors.
    running: BTreeSet<Entity>,
    cooldown: Timer,
    /// A queue of behavior entities that custom implementation asked to perform.
    /// Note that resource availablility check will not run, but this respects overall cooldown.
    /// It will update behavior resources.
    queued: VecDeque<Entity>,
}
impl Default for BehaviorManagerInfo {
    fn default() -> Self {
        Self {
            occupied: Default::default(),
            running: Default::default(),
            cooldown: Timer::from_seconds(1.0, TimerMode::Once),
            queued: Default::default(),
        }
    }
}
impl BehaviorManagerInfo {
    /// Returns all occupied resources and their cooldown time.
    pub fn occupied(&self) -> &BTreeMap<String, Timer> {
        &self.occupied
    }

    /// Clears all running information about the manager.
    pub fn clear(&mut self) {
        self.occupied.clear();
        self.running.clear();
        self.cooldown = Timer::default();
    }

    /// Apply a global cooldown so that the behavior manage don't react too fast
    pub fn set_global_cooldown(&mut self, cooldown: Duration) {
        self.cooldown = Timer::new(cooldown, TimerMode::Once);
    }

    /// Queue a behavior entity to run without checking resource availability, but still respects overall cooldown.
    /// This will prevent auto selection of behaviors until the queue is empty.
    pub fn push_queue(&mut self, entity: Entity) {
        self.queued.push_back(entity);
    }
}

/// Common event for queued and selected behaviors. Precursor of `BehaveStart` which triggers custom code.
#[derive(Event, Debug, Clone, Copy)]
struct BehaviorSelected {
    entity: Entity,
    manager_entity: Entity,
}

fn select_behavior(
    commands: ParallelCommands,
    mut q_manager: Query<(
        Entity,
        &BehaviorManager,
        &mut BehaviorManagerInfo,
        &Children,
    )>,
    q_behavior: Query<(&Behavior, &Weight)>,
    time: Res<Time>,
) {
    use rand::seq::IndexedRandom;

    q_manager
        .par_iter_mut()
        .for_each(|(manager_entity, manager, mut info, children)| {
            if manager.disabled {
                return;
            }

            info.cooldown.tick(time.delta());
            if !info.cooldown.is_finished() {
                return;
            }
            info.cooldown = Default::default();

            // Trigger queued behaviors.
            if let Some(entity) = info.queued.pop_front() {
                commands.command_scope(|mut commands| {
                    commands.trigger(BehaviorSelected {
                        entity,
                        manager_entity,
                    });
                });
                return;
            }

            {
                let mut cooldown_ended = Vec::new();
                for (resource, cooldown) in info.occupied.iter_mut() {
                    cooldown.tick(time.delta());
                    if cooldown.is_finished() {
                        cooldown_ended.push(resource.clone());
                    }
                }
                for resource in cooldown_ended {
                    info.occupied.remove(&resource);
                }
            }

            let mut behaviors = Vec::new();
            for child in children.iter() {
                let Ok((behavior, weight)) = q_behavior.get(child) else {
                    continue;
                };
                // Filter behaviors whose required resources are occupied.
                if info.running.contains(&child)
                    || behavior
                        .occupies
                        .iter()
                        .any(|resource| info.occupied.contains_key(resource))
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

            commands.command_scope(|mut commands| {
                commands.trigger(BehaviorSelected {
                    entity: *entity,
                    manager_entity,
                });
            });
        });
}

fn start_behavior(
    event: On<BehaviorSelected>,
    mut commands: Commands,
    q_behavior: Query<(&Behavior, &Weight)>,
    mut q_manager: Query<(&BehaviorManager, &mut BehaviorManagerInfo, Option<&ChildOf>)>,
) {
    let BehaviorSelected {
        entity,
        manager_entity,
    } = *event;

    let Ok((behavior, _)) = q_behavior.get(entity) else {
        return;
    };
    let Ok((manager, mut info, parent)) = q_manager.get_mut(manager_entity) else {
        return;
    };

    for resource in behavior.occupies.iter() {
        info.occupied.insert(
            resource.to_owned(),
            Timer::from_seconds(99999.0, TimerMode::Once),
        );
    }
    info.running.insert(entity);
    if let Some(cooldown) = behavior.cooldown {
        info.cooldown = Timer::new(cooldown, TimerMode::Once);
    }
    // info!("Triggering behavior {}", behavior.name);
    commands.trigger(BehaveStart {
        entity,
        target: manager
            .target
            .unwrap_or(parent.map(|parent| parent.0).unwrap_or(manager_entity)),
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
    for resource in behavior.occupies.iter() {
        info.occupied.remove(resource);
    }
    for (resource, duration) in event.occupies.iter() {
        info.occupied
            .insert(resource.clone(), Timer::new(*duration, TimerMode::Once));
    }
    info.running.remove(&event.entity);
    let new_duration = info.cooldown.duration() + event.cooldown;
    info.cooldown.set_duration(new_duration);
}

fn intervene_behavior(
    event: On<BehaveIntervene>,
    q_behavior: Query<(), With<Behavior>>,
    mut commands: Commands,
) {
    if q_behavior.get(event.entity).is_ok() {
        commands.trigger(BehaveEnd {
            entity: event.entity,
            cooldown: Default::default(),
            occupies: vec![],
        });
    }
}

fn intervene_behavior_manager(
    mut event: On<BehaveIntervene>,
    mut q_manager: Query<(&mut BehaviorManager, &mut BehaviorManagerInfo, &ChildOf)>,
    mut commands: Commands,
) {
    let Ok((mut manager, mut info, parent)) = q_manager.get_mut(event.entity) else {
        return;
    };
    // Only disables propagation for behavior managers.
    event.propagate(false);
    let target = manager.target.unwrap_or(parent.0);
    for child in info.running.iter() {
        commands.trigger(BehaveIntervene {
            entity: *child,
            target,
        });
    }
    manager.disabled = true;
    info.occupied.clear();
    info.cooldown = Default::default();
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

/// The farther the player is, the more likely this will perform.
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

/// Add a manual multiplier to the behavior weight.
#[derive(Component, Debug)]
pub struct MultiplierManual(pub f32);
impl Default for MultiplierManual {
    fn default() -> Self {
        Self(1.0)
    }
}
fn multiplier_manual(mut q_behavior: Query<(&mut Weight, &MultiplierManual)>) {
    q_behavior
        .par_iter_mut()
        .for_each(|(mut weight, modifier)| {
            weight.multiplier *= modifier.0;
        });
}

/// The faster(or slower) the player is, the more likely this will perform.
///
/// This changes by logarithm.
#[derive(Component, Debug, Default)]
pub struct MultiplierBySpeed {
    pub speed_unit: f32,
    pub speed_unit_log: f32,
    pub log_base: f32,
}
impl MultiplierBySpeed {
    pub fn new(speed_unit: f32, log_base: f32) -> Self {
        Self {
            speed_unit,
            speed_unit_log: speed_unit.log(log_base),
            log_base,
        }
    }
}
fn multiplier_by_speed(
    mut q_behavior: Query<(&mut Weight, &MultiplierBySpeed)>,
    player: Option<Res<crate::player::RandomPlayer>>,
    q_velocity: Query<&LinearVelocity>,
) {
    let Some(player) = player else {
        return;
    };
    let Ok(velocity) = q_velocity.get(player.0) else {
        return;
    };
    let speed = velocity.length();
    q_behavior
        .par_iter_mut()
        .for_each(|(mut weight, modifier)| {
            let result =
                (speed + modifier.speed_unit).log(modifier.log_base) - modifier.speed_unit_log;
            weight.multiplier *= result;
        });
}

/// Applies the corresponding multipliers when the resources are occupied.
#[derive(Component, Debug, Default, Clone)]
pub struct MultiplierWhenResourceOccupied(pub Vec<(String, f32)>);
impl MultiplierWhenResourceOccupied {
    pub fn new<S>(iter: impl IntoIterator<Item = (S, f32)>) -> Self
    where
        S: Into<String>,
    {
        Self(iter.into_iter().map(|(s, w)| (s.into(), w)).collect())
    }
}
fn multiplier_when_resource_occupied(
    mut q_behavior: Query<(&ChildOf, &mut Weight, &MultiplierWhenResourceOccupied)>,
    q_manager_info: Query<&BehaviorManagerInfo>,
) {
    q_behavior
        .par_iter_mut()
        .for_each(|(parent, mut weight, modifier)| {
            let Ok(info) = q_manager_info.get(parent.0) else {
                return;
            };
            for (resource, multiplier) in modifier.0.iter() {
                if info.occupied.contains_key(resource) {
                    weight.multiplier *= *multiplier;
                }
            }
        });
}
