mod prelude;

pub mod math;
pub mod splash;
pub mod transition;

use prelude::*;

pub struct CanumFxPlugin;

impl Plugin for CanumFxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((transition::TransitionPlugin, splash::SplashPlugin));
    }
}
