mod entry;

use canum_play::prelude::*;

pub(super) struct FactoryPlugin;

pub static FACTORY_STATE: LazyLock<setup::Fight> =
    LazyLock::new(|| setup::Fight("Factory".to_owned()));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct FactorySystems;

impl Plugin for FactoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((entry::EntryPlugin,));
        app.configure_sets(
            FixedUpdate,
            FactorySystems.run_if(in_state(FACTORY_STATE.clone())),
        );
    }
}
