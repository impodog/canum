mod base;
mod multiplier;
pub use base::*;
pub use multiplier::*;

use bevy::math::FloatPow;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
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
        app.add_plugins((base::WeightBasePlugin, multiplier::WeightMultiplierPlugin));
        app.init_resource::<PlayerAveragePosition>();
        app.configure_sets(
            FixedPreUpdate,
            (
                BehaviorProcess::Init,
                BehaviorProcess::Calc,
                BehaviorProcess::Trigger,
            )
                .chain(),
        );
        app.add_systems(
            FixedPreUpdate,
            (
                init_behavior_weights,
                update_player_average_position,
                update_manager_info,
            )
                .in_set(BehaviorProcess::Init),
        );
        app.add_systems(
            FixedPreUpdate,
            select_behavior.in_set(BehaviorProcess::Trigger),
        );

        app.add_observer(start_behavior)
            .add_observer(end_behavior)
            .add_observer(intervene_behavior)
            .add_observer(push_queue_behavior);
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
    /// Some the behavior is queued by another certain name.
    pub caller: Option<String>,
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
    /// Maps from behavior name to entity.
    name_map: HashMap<String, Entity>,
    occupied: BTreeMap<String, Timer>,
    /// All running behaviors.
    running: BTreeSet<Entity>,
    cooldown: Timer,
    /// A queue of behavior entities that custom implementation asked to perform.
    /// Note that resource availablility check will not run, but this respects overall cooldown.
    /// It will update behavior resources.
    queued: VecDeque<(Entity, String)>,
}
impl Default for BehaviorManagerInfo {
    fn default() -> Self {
        Self {
            name_map: Default::default(),
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
    fn push_queue(&mut self, entity: Entity, caller: impl Into<String>) {
        self.queued.push_back((entity, caller.into()));
    }
}

/// Call this on behaviors, for their manager to queue another behavior.
/// This will prevent auto selection of behaviors until the queue is empty.
#[derive(EntityEvent, Debug, Clone)]
pub struct BehaveQueue {
    /// The caller behavior entity.
    pub entity: Entity,
    /// The name of the target behavior.
    pub name: String,
}
impl BehaveQueue {
    pub fn new(entity: Entity, name: impl Into<String>) -> Self {
        Self {
            entity,
            name: name.into(),
        }
    }
}

fn update_manager_info(
    mut q_manager: Query<(&mut BehaviorManagerInfo, &Children), With<BehaviorManager>>,
    q_behavior: Query<&Behavior>,
) {
    q_manager.par_iter_mut().for_each(|(mut info, children)| {
        for child in children.iter() {
            let Ok(behavior) = q_behavior.get(child) else {
                continue;
            };
            if info
                .name_map
                .get(&behavior.name)
                .is_none_or(|old_child| *old_child != child)
            {
                info.name_map.insert(behavior.name.clone(), child);
            }
        }
    });
}

fn push_queue_behavior(
    event: On<BehaveQueue>,
    mut q_manager: Query<&mut BehaviorManagerInfo>,
    q_behavior: Query<(&Behavior, &ChildOf)>,
) {
    let Ok((behavior, parent)) = q_behavior.get(event.entity) else {
        return;
    };
    let Ok(mut manager) = q_manager.get_mut(parent.0) else {
        return;
    };
    if let Some(target) = manager.name_map.get(&event.name).copied() {
        manager.push_queue(target, behavior.name.clone());
    } else {
        warn!(
            "Unable to find requested sibling behavior {} for behavior {}.",
            event.name, behavior.name
        );
    }
}

/// Common event for queued and selected behaviors. Precursor of `BehaveStart` which triggers custom code.
#[derive(Event, Debug, Clone)]
struct BehaviorSelected {
    entity: Entity,
    manager_entity: Entity,
    caller: Option<String>,
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
            if let Some((entity, caller)) = info.queued.pop_front() {
                commands.command_scope(|mut commands| {
                    commands.trigger(BehaviorSelected {
                        entity,
                        manager_entity,
                        caller: Some(caller),
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
                    caller: None,
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
        caller,
    } = event.clone();

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
        caller,
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
