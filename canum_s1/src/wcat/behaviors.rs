pub mod bar;
pub mod phase1;

use super::*;
use canum_play::enemy::behavior::*;

pub(super) struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((bar::BarPlugin, phase1::Phase1Plugin));
        app.add_systems(OnEnter(WCAT_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(initialize_behaviors)));
        });
    }
}

fn initialize_behaviors(
    _event: On<entry::WcatFightStart>,
    mut commands: Commands,
    q_wcat: Query<Entity, With<WcatBoss>>,
) {
    let Ok(wcat) = q_wcat.single() else {
        return;
    };
    commands.entity(wcat).insert(phase1::WcatPhase1);
}
