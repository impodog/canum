use super::*;
use enemy::behavior::*;

pub(super) struct AttacksPlugin;

impl Plugin for AttacksPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component, Default)]
#[require(BehaviorManager)]
pub struct LionBehaviors;
