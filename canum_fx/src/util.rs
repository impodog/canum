use crate::prelude::*;

pub(super) struct UtilPlugin;

impl Plugin for UtilPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPreUpdate, (update_wait, check_despawn));
    }
}

/// Wait after an interval, and then notify the specified Entity(defaults to self), and in the next frame despawn itself.
#[derive(Component, Default, Debug, Clone)]
pub struct WaitInterval {
    pub timer: Timer,
    pub notify: Option<Entity>,
}
impl WaitInterval {
    pub fn new(duration: std::time::Duration) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Once),
            notify: None,
        }
    }

    pub fn notify(duration: std::time::Duration, entity: Entity) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Once),
            notify: Some(entity),
        }
    }
}

#[derive(EntityEvent, Debug)]
pub struct WaitComplete {
    pub entity: Entity,
}

fn update_wait(
    commands: ParallelCommands,
    mut q_wait: Query<(Entity, &mut WaitInterval)>,
    time: Res<Time>,
) {
    q_wait.par_iter_mut().for_each(|(entity, mut wait)| {
        if wait.timer.is_finished() {
            commands.command_scope(|mut commands| {
                commands.entity(entity).despawn();
            });
        } else {
            wait.timer.tick(time.delta());
            if wait.timer.just_finished() {
                commands.command_scope(|mut commands| {
                    let notify = wait.notify.unwrap_or(entity);
                    commands.trigger(WaitComplete { entity: notify });
                });
            }
        }
    });
}

/// Creates a type that when spawn, wait for a interval and triggers a specific event(using Default::default()).
/// You must also add its observer `Self::observer`
#[macro_export]
macro_rules! wait_then_trigger {
    ($name: ident, $type: ty, $interval: expr) => {
        #[derive(Component, Default)]
        #[require($crate::util::WaitInterval::new(Duration::from_secs_f32($interval)))]
        struct $name;
        impl $name {
            fn observer(_event: On<$crate::util::WaitComplete>, mut commands: Commands) {
                commands.trigger(<$type>::default());
            }
        }
    };

    ($name: ident, $type: ty, $data: ty, $interval: expr) => {
        impl $type {
            fn new(value: $data) -> Self {
                Self(value)
            }
        }

        #[derive(Component)]
        #[require($crate::util::WaitInterval::new(Duration::from_secs_f32($interval)))]
        struct $name($data);
        impl $name {
            fn observer(
                event: On<$crate::util::WaitComplete>,
                mut commands: Commands,
                query: Query<&$name>,
            ) {
                let Ok(value) = query.get(event.entity) else {
                    return;
                };
                commands.trigger(<$type>::new(value.0.clone()));
            }
        }
    };
}

#[macro_export]
macro_rules! session_observers {
    ($commands:expr, $($fn:expr),* $(,)?) => {{
            $commands.spawn_batch([
                $((canum_play::setup::SessionOnly, bevy::prelude::Observer::new($fn))),*
            ]);
    }};
}

/// In many canum_fx utilities, choose the behavior when the target entity is despawned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DespawnBehavior {
    #[default]
    Despawn,
    Remove,
}

/// Notify an entity(default self) when the target entity is despawned.
#[derive(Component, Debug, Clone, Copy)]
#[non_exhaustive]
pub struct DespawnCheck {
    pub target: Entity,
    pub notify: Option<Entity>,
    pub despawn_behavior: DespawnBehavior,
}
impl DespawnCheck {
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            notify: None,
            despawn_behavior: default(),
        }
    }
    pub fn with_notify(mut self, notify: Entity) -> Self {
        self.notify = Some(notify);
        self
    }
    pub fn with_despawn_behavior(mut self, despawn_behavior: DespawnBehavior) -> Self {
        self.despawn_behavior = despawn_behavior;
        self
    }
}

/// Notify event of `DespawnCheck`.
#[derive(EntityEvent, Debug)]
pub struct DespawnObserved {
    pub entity: Entity,
}

fn check_despawn(
    q_until_despawn: Query<(Entity, &DespawnCheck)>,
    q_entity: Query<()>,
    commands: ParallelCommands,
) {
    q_until_despawn
        .par_iter()
        .for_each(|(checker_entity, checker)| {
            if q_entity.get(checker.target).is_err() {
                if let Some(notify) = checker.notify {
                    commands.command_scope(|mut commands| {
                        commands.trigger(DespawnObserved { entity: notify });
                    });
                } else {
                    commands.command_scope(|mut commands| {
                        commands.trigger(DespawnObserved {
                            entity: checker_entity,
                        });
                    });
                }
                match checker.despawn_behavior {
                    DespawnBehavior::Despawn => {
                        commands.command_scope(|mut commands| {
                            commands.entity(checker_entity).despawn();
                        });
                    }
                    DespawnBehavior::Remove => {
                        commands.command_scope(|mut commands| {
                            commands.entity(checker_entity).remove::<DespawnCheck>();
                        });
                    }
                }
            }
        });
}
