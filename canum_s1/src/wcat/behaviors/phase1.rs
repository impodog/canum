use super::*;

pub(super) struct Phase1Plugin;

impl Plugin for Phase1Plugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component, Debug, Clone)]
#[require(BehaviorManager::new())]
pub struct WcatPhase1;
