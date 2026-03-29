mod background;

use crate::prelude::*;

pub(super) struct TransitionPlugin;

impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((background::BackgroundPlugin,));
    }
}
