mod filed;

use crate::prelude::*;
use canum_play::player::attack::*;
use canum_play::player::*;

pub(super) struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((filed::FiledPlugin,));
        app.add_observer(add_weapons);
    }
}

fn fmul(first: i32, second: f32) -> i32 {
    (first as f32 * second) as i32
}

fn add_weapons(event: On<setup::PostStartSession>, mut commands: Commands, save: Res<Save>) {
    let mut weapons = Vec::new();
    let mut damage_multiplier: f32 = 1.0;
    for effect in save.progress.selected_effects.iter() {
        if let Some(value) = effect.strip_prefix("Damage%") {
            match value.parse::<i32>() {
                Ok(percent) => {
                    damage_multiplier += percent as f32 / 100.0;
                }
                Err(err) => {
                    error!("Invalid damage multiplier effect: {value}, {err}");
                }
            }
        }
    }
    for weapon in save.progress.selected_weapons.iter() {
        match weapon.as_str() {
            "Filed" => {
                let mut filed = filed::Filed::default();
                filed.damage = fmul(filed.damage, damage_multiplier);
                weapons.push(Some(
                    commands.spawn((ChildOf(event.player_entity), filed)).id(),
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
