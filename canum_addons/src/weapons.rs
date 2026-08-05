mod filed;
mod spread;

use crate::prelude::*;
use canum_play::player::attack::*;
use canum_play::player::*;

pub(super) struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((filed::FiledPlugin, spread::SpreadPlugin));
        app.add_observer(init_weapon_values)
            .add_observer(apply_weapon_effects)
            .add_observer(spawn_weapons);
    }
}

#[derive(Resource)]
pub struct WeaponArguments {
    pub damage_addition: f64,
    pub damage_multiplier: f64,
}
impl Default for WeaponArguments {
    fn default() -> Self {
        Self {
            damage_addition: 0.0,
            damage_multiplier: 1.0,
        }
    }
}

fn init_weapon_values(_event: On<setup::StartSessionFirst>, mut commands: Commands) {
    commands.insert_resource(WeaponArguments::default());
}

fn apply_weapon_effects(
    _event: On<setup::StartSessionMiddle>,
    save: Res<Save>,
    mut args: ResMut<WeaponArguments>,
) {
    for effect in save
        .progress
        .selected_effects
        .range_starting_with("Damage+%")
    {
        if let Some(value) = effect.strip_prefix("Damage+%") {
            match value.parse::<i32>() {
                Ok(percent) => {
                    args.damage_addition += percent as f64 / 100.0;
                }
                Err(err) => {
                    error!("Invalid damage addition effect: {value}, {err}");
                }
            }
        }
    }
}

fn spawn_weapons(
    event: On<setup::StartSessionAction>,
    mut commands: Commands,
    save: Res<Save>,
    args: Res<WeaponArguments>,
) {
    let mut weapons = Vec::new();

    let total_damage_multiplier = (1.0 + args.damage_addition) * args.damage_multiplier;

    for weapon in save.progress.selected_weapons.iter() {
        match weapon.as_str() {
            "A_Filed" => {
                let mut filed = filed::Filed::default();
                filed.damage = filed.damage.mul(total_damage_multiplier);
                weapons.push(Some(
                    commands.spawn((ChildOf(event.player_entity), filed)).id(),
                ));
            }
            "B_Spread" => {
                let mut spread = spread::Spread::default();
                spread.damage = spread.damage.mul(total_damage_multiplier);
                weapons.push(Some(
                    commands.spawn((ChildOf(event.player_entity), spread)).id(),
                ));
            }
            _ => {
                weapons.push(None);
            }
        }
    }
    commands
        .entity(event.player_entity)
        .insert(Weapons(weapons));
}
