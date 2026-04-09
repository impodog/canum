//! Tracks the feats the player has completed.

use crate::prelude::*;

pub(super) struct TrackingPlugin;

impl Plugin for TrackingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_tracking)
            .add_observer(update_any_hits)
            .add_observer(gain_coins);
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct AnyHits(pub bool);
fn update_any_hits(_event: On<super::health::ActuallyHit>, mut any_hits: ResMut<AnyHits>) {
    any_hits.0 = true;
}

fn init_tracking(_event: On<crate::setup::PostStartSession>, mut commands: Commands) {
    commands.insert_resource(AnyHits::default());
}

fn gain_coins(
    event: On<super::victory::CompletedTasks>,
    fight: Res<State<crate::setup::Fight>>,
    mut save: ResMut<Save>,
) {
    info!("Completed: {:?}", event);
    if let Some(details) = CONFIG.values.boss.get(&fight.get().0) {
        for task in event.iter() {
            if let Some(coins) = details.gains.get(task) {
                save.progress.coins += *coins;
            }
        }
    }
}
