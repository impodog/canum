use crate::prelude::*;
use canum_ui::health::HealthDetails;

pub(super) struct HealthsPlugin;

impl Plugin for HealthsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_health_values)
            .add_observer(apply_health_effects)
            .add_observer(update_health_entity);
    }
}

fn init_health_values(
    _event: On<setup::StartSessionFirst>,
    mut commands: Commands,
    save: Res<Save>,
) {
    match save.progress.selected_health.as_str() {
        "BasicHp" => {
            commands.insert_resource(HealthDetails::BasicHp(default()));
        }
        _ => {
            warn!(
                "Unknown health selection {}, using default BasicHp.",
                save.progress.selected_health
            );
            commands.insert_resource(HealthDetails::BasicHp(default()));
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
            let (value, image) = value.split_once(':').unwrap_or((value, "Hp_BasicHp"));
            match value.parse::<isize>() {
                Ok(add) => match health.as_mut() {
                    HealthDetails::BasicHp(basic_hp) => {
                        for _ in 0..add {
                            basic_hp.array.push(image.to_owned());
                        }
                    }
                },
                Err(err) => {
                    error!("Invalid health addition effect: {value}, {err}");
                }
            }
        }
    }
}

fn update_health_entity(
    _event: On<setup::StartSessionAction>,
    health: Res<HealthDetails>,
    mut q_integer_health: Query<&mut player::health::IntegerHealth>,
    primary_player: Res<player::PrimaryPlayer>,
) {
    match health.as_ref() {
        HealthDetails::BasicHp(basic_hp) => {
            let Ok(mut integer_health) = q_integer_health.get_mut(primary_player.0) else {
                return;
            };
            integer_health.count = basic_hp.count() as i32;
        }
    }
}
