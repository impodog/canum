use crate::prelude::*;

mod entry;
mod running;

pub(super) struct RunwayPlugin;

static RUNWAY_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Runway".to_owned()));

impl Plugin for RunwayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((entry::EntryPlugin, running::RunningPlugin));
    }
}

/// This only marks if the level(displacement based) has been completed.
#[derive(Component, Debug)]
#[require(health::Friendly(false), player::victory::DefeatToWin::default())]
pub struct RunwayBoss {
    pub timer: Timer,
}

/// This is the main enemy. Although they only run side by side with player, and occasionally attack.
#[derive(Component, Debug)]
#[require(Animation, SessionOnly)]
pub struct RunwayLion;
