use crate::prelude::*;
use canum_ui::health::HealthDetails;

pub(super) struct HealthsPlugin;

impl Plugin for HealthsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_health_values)
            .add_observer(apply_health_effects);
    }
}

fn init_health_values(
    _event: On<setup::StartSessionFirst>,
    mut commands: Commands,
    save: Res<Save>,
) {
    match save.progress.selected_health.as_str() {
        "BasicHp" => {
            commands.insert_resource(HealthDetails::BasicHp(6));
        }
        _ => {
            warn!(
                "Unknown health selection {}, using default BasicHp.",
                save.progress.selected_health
            );
            commands.insert_resource(HealthDetails::BasicHp(6));
        }
    }
}

fn apply_health_effects(
    _event: On<setup::StartSessionMiddle>,
    mut health: ResMut<HealthDetails>,
    save: Res<Save>,
) {
    for effect in save
        .progress
        .selected_effects
        .range_starting_with("Health+")
    {
        if let Some(value) = effect.strip_prefix("Health+") {
            match value.parse::<isize>() {
                Ok(add) => match health.as_mut() {
                    HealthDetails::BasicHp(number) => {
                        *number = number.saturating_add_signed(add);
                    }
                },
                Err(err) => {
                    error!("Invalid health addition effect: {value}, {err}");
                }
            }
        }
    }
}
