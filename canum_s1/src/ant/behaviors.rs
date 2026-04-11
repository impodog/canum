use crate::prelude::*;
use enemy::behavior::*;

pub(super) struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(super::ANT_STATE.clone()),
            |mut commands: Commands| {
                commands.insert_resource(AntStage::default());
            },
        );
        app.add_systems(
            OnExit(super::ANT_STATE.clone()),
            |mut commands: Commands| {
                commands.remove_resource::<AntStage>();
            },
        );
    }
}

/// Stores stage-related values.
#[derive(Resource, Debug)]
pub struct AntStage {
    pub stage: u8,
}

impl Default for AntStage {
    fn default() -> Self {
        Self { stage: 1 }
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct AntBehaviors;

#[derive(Component, Default)]
#[require(Behavior::new("Ant_RunToPlayer", 1.5, ["RunToPlayer", "Displacement"]))]
struct RunToPlayer;
