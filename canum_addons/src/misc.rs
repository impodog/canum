pub mod weapon_indic;

use super::*;

pub(super) struct MiscPlugin;

impl Plugin for MiscPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((weapon_indic::WeaponIndicPlugin,));
    }
}
