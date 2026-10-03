mod s1;
mod s2;

use crate::prelude::*;

pub(super) struct CharmsPlugin;

impl Plugin for CharmsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((s1::S1Plugin, s2::S2Plugin));
    }
}
