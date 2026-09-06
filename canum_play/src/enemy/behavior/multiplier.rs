use super::*;

pub(super) struct WeightMultiplierPlugin;

impl Plugin for WeightMultiplierPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (
                multiplier_manual,
                multiplier_by_speed,
                multiplier_when_resource_occupied,
            )
                .in_set(BehaviorProcess::Calc),
        );
    }
}

/// Add a manual multiplier to the behavior weight.
#[derive(Component, Debug)]
pub struct MultiplierManual(pub f32);
impl Default for MultiplierManual {
    fn default() -> Self {
        Self(1.0)
    }
}
fn multiplier_manual(mut q_behavior: Query<(&mut Weight, &MultiplierManual)>) {
    q_behavior
        .par_iter_mut()
        .for_each(|(mut weight, modifier)| {
            weight.multiplier *= modifier.0;
        });
}

/// The faster(or slower) the player is, the more likely this will perform.
///
/// This changes by logarithm.
#[derive(Component, Debug, Default)]
pub struct MultiplierBySpeed {
    pub speed_unit: f32,
    pub speed_unit_log: f32,
    pub log_base: f32,
}
impl MultiplierBySpeed {
    pub fn new(speed_unit: f32, log_base: f32) -> Self {
        Self {
            speed_unit,
            speed_unit_log: speed_unit.log(log_base),
            log_base,
        }
    }
}
fn multiplier_by_speed(
    mut q_behavior: Query<(&mut Weight, &MultiplierBySpeed)>,
    player: Option<Res<crate::player::RandomPlayer>>,
    q_velocity: Query<&LinearVelocity>,
) {
    let Some(player) = player else {
        return;
    };
    let Ok(velocity) = q_velocity.get(player.0) else {
        return;
    };
    let speed = velocity.length();
    q_behavior
        .par_iter_mut()
        .for_each(|(mut weight, modifier)| {
            let result =
                (speed + modifier.speed_unit).log(modifier.log_base) - modifier.speed_unit_log;
            weight.multiplier *= result;
        });
}

/// Applies the corresponding multipliers when the resources are occupied.
#[derive(Component, Debug, Default, Clone)]
pub struct MultiplierWhenResourceOccupied(pub Vec<(String, f32)>);
impl MultiplierWhenResourceOccupied {
    pub fn new<S>(iter: impl IntoIterator<Item = (S, f32)>) -> Self
    where
        S: Into<String>,
    {
        Self(iter.into_iter().map(|(s, w)| (s.into(), w)).collect())
    }
}
fn multiplier_when_resource_occupied(
    mut q_behavior: Query<(&ChildOf, &mut Weight, &MultiplierWhenResourceOccupied)>,
    q_manager_info: Query<&BehaviorManagerInfo>,
) {
    q_behavior
        .par_iter_mut()
        .for_each(|(parent, mut weight, modifier)| {
            let Ok(info) = q_manager_info.get(parent.0) else {
                return;
            };
            for (resource, multiplier) in modifier.0.iter() {
                if info.occupied.contains_key(resource) {
                    weight.multiplier *= *multiplier;
                }
            }
        });
}
