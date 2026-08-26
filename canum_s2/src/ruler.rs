mod entry;

use super::*;

pub static RULER_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Ruler".to_owned()));
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct RulerSet;

pub(super) struct RulerPlugin;

impl Plugin for RulerPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(FixedUpdate, RulerSet.run_if(in_state(RULER_STATE.clone())));
        app.add_plugins((entry::EntryPlugin,));
    }
}
