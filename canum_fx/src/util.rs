use crate::prelude::*;

pub(super) struct UtilPlugin;

impl Plugin for UtilPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPreUpdate, update_wait);
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
