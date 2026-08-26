pub mod prelude;
pub mod weapon;

use prelude::*;

pub struct CanumToolPlugin;

impl Plugin for CanumToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((weapon::WeaponPlugin,));
    }
}
