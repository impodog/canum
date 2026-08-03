//! Tracks the feats the player has completed.

use crate::prelude::*;

pub(super) struct TrackingPlugin;

impl Plugin for TrackingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_tracking)
            .add_observer(update_any_hits)
            .add_observer(gain_coins)
            .add_observer(update_stage);
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct AnyHits(pub bool);
fn update_any_hits(_event: On<super::health::ActuallyHit>, mut any_hits: ResMut<AnyHits>) {
    any_hits.0 = true;
}

fn init_tracking(event: On<crate::setup::StartSessionFirst>, mut commands: Commands) {
    // States starting with '_' are special in-between states.
    if !event.fight.starts_with('_') {
        commands.insert_resource(AnyHits::default());
    }
}

fn gain_coins(event: On<super::victory::CompletedTasks>, mut save: ResMut<Save>) {
    info!("Completed: {:?}", event);
    if let Some(details) = CONFIG.values.boss.get(&event.fight) {
        for task in event.iter() {
            if let Some(coins) = details.gains.get(task) {
                save.progress.coins += *coins;
            }
        }
    }
}

fn update_stage(event: On<super::victory::CompletedTasks>, mut save: ResMut<Save>) {
    for (stage, details) in CONFIG.values.stage.iter() {
        if !details.complete_prereqs.contains(&event.fight) {
            continue;
        }
        if save.progress.completed_stages.contains(stage) {
            continue;
        }
        let completed = details.complete_prereqs.iter().all(|boss| {
            save.progress
                .boss_progress
                .get(boss)
                .is_some_and(|progress| progress.defeated)
        });
        if completed {
            save.progress.completed_stages.insert(stage.clone());
        }
    }
}
