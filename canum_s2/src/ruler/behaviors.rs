mod laser;
pub mod phase1;
pub mod phase2;
pub mod transition;

use super::*;
use enemy::behavior::*;

pub(super) struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            phase1::Phase1Plugin,
            laser::LaserPlugin,
            phase2::Phase2Plugin,
            transition::TransitionPlugin,
        ));
    }
}
