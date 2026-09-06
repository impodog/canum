use super::*;

pub(super) struct WeightBasePlugin;

impl Plugin for WeightBasePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (base_by_distance, base_farther_better).in_set(BehaviorProcess::Calc),
        );
    }
}

/// Decides weight base by distance to the player.
/// This uses gaussian probability.
#[derive(Component, Debug, Default)]
#[require(Transform)]
pub struct BaseByDistance {
    pub mean: f32,
    pub deviation: f32,
}
impl BaseByDistance {
    pub fn new(mean: f32, deviation: f32) -> Self {
        Self { mean, deviation }
    }
}
fn base_by_distance(
    mut q_behavior: Query<(&mut Weight, &BaseByDistance, &GlobalTransform)>,
    average_position: Res<PlayerAveragePosition>,
) {
    q_behavior
        .par_iter_mut()
        .for_each(|(mut weight, modifier, global_transform)| {
            let distance = global_transform
                .translation()
                .xy()
                .distance(**average_position);
            weight.base = std::f32::consts::E
                .powf(-(distance - modifier.mean).squared() * 0.5 / modifier.deviation.squared());
        });
}

/// The farther the player is, the more likely this will perform.
/// This uses inverse function.
#[derive(Component, Debug, Default)]
#[require(Transform)]
pub struct BaseFartherBetter {
    /// The probability increases from 0.0, after distance > `start`.
    pub start: f32,
    pub unit_length: f32,
}
impl BaseFartherBetter {
    pub fn new(start: f32, unit_length: f32) -> Self {
        Self { start, unit_length }
    }
}
fn base_farther_better(
    mut q_behavior: Query<(&mut Weight, &BaseFartherBetter, &GlobalTransform)>,
    average_position: Res<PlayerAveragePosition>,
) {
    q_behavior
        .par_iter_mut()
        .for_each(|(mut weight, modifier, global_transform)| {
            let distance = global_transform
                .translation()
                .xy()
                .distance(**average_position);
            if distance >= modifier.start {
                weight.base =
                    1.0 - 1.0 / ((distance - modifier.start) / modifier.unit_length + 1.0);
            } else {
                weight.base = 0.0;
            }
        });
}
