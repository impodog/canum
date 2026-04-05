mod background;
mod behaviors;
mod defeat;

use std::sync::LazyLock;

use crate::prelude::*;

static TURF_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Turf".to_owned()));

pub(super) struct TurfPlugin;

impl Plugin for TurfPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            background::BackgroundPlugin,
            behaviors::BehaviorsPlugin,
            defeat::DefeatPlugin,
        ));
        app.add_systems(
            FixedPreUpdate,
            turf_count_down.run_if(in_state(TURF_STATE.clone())),
        );
    }
}

/// This is a timer count-down boss, therefore it has no health component.
#[derive(Component, Debug)]
#[require(health::Friendly(false), player::victory::DefeatToWin::default())]
pub struct TurfBoss {
    pub timer: Timer,
}
impl Default for TurfBoss {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(60.0, TimerMode::Once),
        }
    }
}

fn turf_count_down(
    mut q_turf: Query<(&mut TurfBoss, &mut player::victory::DefeatToWin)>,
    time: Res<Time<Real>>,
) {
    for (mut turf, mut defeat_to_win) in q_turf.iter_mut() {
        turf.timer.tick(time.delta());
        defeat_to_win.defeated = true;
    }
}
