mod prelude;

pub mod emphasis;
pub mod math;
pub mod splash;
pub mod text;
pub mod transform;
pub mod transition;
pub mod util;
pub mod visual;

use prelude::*;

pub struct CanumFxPlugin;

impl Plugin for CanumFxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            transition::TransitionPlugin,
            splash::SplashPlugin,
            util::UtilPlugin,
            text::TextPlugin,
            visual::VisualPlugin,
            emphasis::EmphasisPlugin,
            transform::TransformPlugin,
        ));
    }
}
