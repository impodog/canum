use super::*;
use enemy::behavior::*;

pub(super) struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct RobotBehaviors;

#[derive(Component, Default)]
#[require(Behavior::new("Factory_Robot_DashAway", 0.2, ["Move", "DashAway"]))]
pub struct DashAway;

fn start_dash_away(event: On<BehaveStart>) {}
