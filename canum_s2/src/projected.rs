mod entry;
mod flashlight;

pub static PROJECTED_STATE: LazyLock<setup::Fight> =
    LazyLock::new(|| setup::Fight("Projected".to_owned()));

use crate::prelude::*;

pub(super) struct ProjectedPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, SystemSet)]
pub struct ProjectedSet;

impl Plugin for ProjectedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((flashlight::FlashlightPlugin, entry::EntryPlugin));
        app.configure_sets(
            FixedUpdate,
            ProjectedSet.run_if(in_state(PROJECTED_STATE.to_owned())),
        );
    }
}
