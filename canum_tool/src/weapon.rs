pub mod laser;
pub mod old_laser;

use crate::prelude::*;

pub(super) struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((old_laser::OldLaserPlugin, laser::LaserPlugin));
    }
}
