use crate::*;

static LASER_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Laser".to_owned()));

pub(super) struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {}
}
