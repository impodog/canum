use std::collections::VecDeque;

use super::menu::*;
use super::*;

pub(super) struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, update_weapon_used);
        app.add_observer(update_weapon_select);
    }
}

/// Makes sure the menu is weapon select menu.
#[derive(Component, Default)]
pub struct WeaponSelectMenu;

fn update_weapon_select(
    event: On<SelectInput>,
    mut q_menu: Query<(Entity, &mut SelectMenu)>,
    q_weapon_select: Query<(), With<WeaponSelectMenu>>,
    mut commands: Commands,
    save: Res<Save>,
) {
    if q_weapon_select.single().is_err() {
        return;
    }
    let Ok((entity, mut menu)) = q_menu.single_mut() else {
        return;
    };
    if save.progress.gained_weapons.is_empty() {
        return;
    }
    if menu.options.is_empty() {
        for charm in save.progress.gained_weapons.iter().take(SELECT_MENU_LENGTH) {
            menu.options.push_back(charm.to_owned());
        }
    }
    match *event {
        SelectInput::Next => {
            let next_element =
                next_cyclic(&save.progress.gained_weapons, menu.options.back().unwrap()).clone();
            menu.options.pop_front();
            menu.options.push_back(next_element);
        }
        SelectInput::Prev => {
            let prev_element =
                prev_cyclic(&save.progress.gained_weapons, menu.options.front().unwrap()).clone();
            menu.options.pop_back();
            menu.options.push_front(prev_element);
        }
        SelectInput::NextPage => {
            let next_elements = next_n_cyclic(
                &save.progress.gained_weapons,
                menu.options.back().unwrap(),
                SELECT_MENU_LENGTH,
            )
            .cloned()
            .collect::<VecDeque<_>>();
            menu.options = next_elements;
        }
        SelectInput::PrevPage => {
            let prev_elements = prev_n_cyclic(
                &save.progress.gained_weapons,
                menu.options.front().unwrap(),
                SELECT_MENU_LENGTH,
            )
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<VecDeque<_>>();
            menu.options = prev_elements;
        }
        SelectInput::Update => {}
        SelectInput::Toggle => {
            let Some(selected) = menu.options.front() else {
                return;
            };
            commands.trigger(UpdateWeapons::Toggle(selected.clone()));
        }
    }

    commands.trigger(super::menu::UpdateOptionsDisplay { entity });
}

fn update_weapon_used(
    mut q_charm: Query<(&mut ImageNode, &super::menu::OptionName)>,
    save: Res<Save>,
) {
    q_charm
        .par_iter_mut()
        .for_each(|(mut image_node, option_name)| {
            if let Some(weapon) = option_name.0.strip_prefix("Weapon_") {
                if save
                    .progress
                    .selected_weapons
                    .iter()
                    .find(|value| **value == *weapon)
                    .is_some()
                {
                    image_node.color.set_alpha(0.2);
                } else {
                    image_node.color.set_alpha(1.0);
                }
            }
        });
}
