use crate::prelude::*;

mod defeat;
mod entry;
mod obstacles;
mod problem;
mod running;

pub(super) struct RunwayPlugin;

static RUNWAY_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Runway".to_owned()));

impl Plugin for RunwayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            entry::EntryPlugin,
            running::RunningPlugin,
            obstacles::ObstaclesPlugin,
            problem::ProblemPlugin,
            defeat::DefeatPlugin,
        ));
    }
}

/// This only marks if the level(displacement based) has been completed.
#[derive(Component, Debug)]
#[require(
    SessionOnly,
    health::Friendly(false),
    player::victory::DefeatToWin::default()
)]
pub struct RunwayBoss;

/// This is the main enemy. Although they only run side by side with player, and occasionally attack.
#[derive(Component, Default)]
#[require(
    Animation,
    RigidBody::Dynamic,
    Collider::rectangle(40.0, 50.0),
    health::Friendly(false),
    SessionOnly,
    movements::ForcedVelocity
)]
pub struct RunwayLion;

#[derive(Component, Default)]
#[require(SessionOnly, obstacles::ObstacleBehaviors)]
pub struct RunwayObstacles;
