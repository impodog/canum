use std::collections::VecDeque;

use super::{menu::*, *};

pub(super) struct CharmsPlugin;

impl Plugin for CharmsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPostUpdate,
            (update_charm_used, update_charm_requirements),
        );
        app.add_observer(update_charm_select)
            .add_observer(clear_indicators);
    }
}

/// Makes sure the menu is charm select menu.
#[derive(Component, Default)]
pub struct CharmSelectMenu;

fn update_charm_select(
    event: On<SelectInput>,
    mut q_menu: Query<(Entity, &mut SelectMenu)>,
    q_charm_select: Query<(), With<CharmSelectMenu>>,
    mut commands: Commands,
    save: Res<Save>,
) {
    if q_charm_select.single().is_err() {
        return;
    }
    let Ok((entity, mut menu)) = q_menu.single_mut() else {
        return;
    };
    if save.progress.gained_charms.is_empty() {
        // TODO: Maybe indicator for no charms?
        return;
    }
    if menu.options.is_empty() {
        for charm in save.progress.gained_charms.iter().take(SELECT_MENU_LENGTH) {
            menu.options.push_back(charm.to_owned());
        }
    }
    match *event {
        SelectInput::Next => {
            let next_element =
                next_cyclic(&save.progress.gained_charms, menu.options.back().unwrap()).clone();
            menu.options.pop_front();
            menu.options.push_back(next_element);
        }
        SelectInput::Prev => {
            let prev_element =
                prev_cyclic(&save.progress.gained_charms, menu.options.front().unwrap()).clone();
            menu.options.pop_back();
            menu.options.push_front(prev_element);
        }
        SelectInput::NextPage => {
            let next_elements = next_n_cyclic(
                &save.progress.gained_charms,
                menu.options.back().unwrap(),
                SELECT_MENU_LENGTH,
            )
            .cloned()
            .collect::<VecDeque<_>>();
            menu.options = next_elements;
        }
        SelectInput::PrevPage => {
            let prev_elements = prev_n_cyclic(
                &save.progress.gained_charms,
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
            commands.trigger(UpdateCharms::Toggle(selected.clone()));
        }
    }

    commands.trigger(super::menu::UpdateOptionsDisplay { entity });
}

fn update_charm_used(
    mut q_charm: Query<(&mut ImageNode, &super::menu::OptionName)>,
    save: Res<Save>,
) {
    q_charm
        .par_iter_mut()
        .for_each(|(mut image_node, option_name)| {
            if let Some(charm) = option_name.0.strip_prefix("Charm_") {
                if save.progress.charms.selected_charms.contains(charm) {
                    image_node.color.set_alpha(0.2);
                } else {
                    image_node.color.set_alpha(1.0);
                }
            }
        });
}

#[derive(Event)]
struct ClearRequirementIndicators;

fn update_charm_requirements(
    q_menu: Query<&SelectMenu>,
    mut q_indicator: Query<(&mut Visibility, &CharmOccupiesIndicator)>,
    q_charm_select: Query<(), With<CharmSelectMenu>>,
    mut commands: Commands,
) {
    // First check if is in equipment mode.
    let Ok(menu) = q_menu.single() else {
        return;
    };
    if q_charm_select.single().is_err() {
        commands.trigger(ClearRequirementIndicators);
        return;
    };
    let Some(current) = menu.options.front() else {
        commands.trigger(ClearRequirementIndicators);
        return;
    };
    if let Some(details) = CONFIG.values.charm.get(current) {
        // This shows a corresponding indicator if available, or it will show the general slot.
        let mut ok = false;
        for (mut visibility, indicator) in q_indicator.iter_mut() {
            if (indicator.slot.to_cost().bitset() & details.cost.bitset()) != 0 {
                *visibility = Visibility::Inherited;
                ok = true;
                break;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
        if !ok {
            for (mut visibility, indicator) in q_indicator.iter_mut() {
                if indicator.slot == Slot::General {
                    *visibility = Visibility::Inherited;
                    break;
                }
            }
        }
    } else {
        commands.trigger(ClearRequirementIndicators);
    }
}

fn clear_indicators(
    _event: On<ClearRequirementIndicators>,
    mut q_indicator: Query<&mut Visibility, With<CharmOccupiesIndicator>>,
) {
    for mut visibility in q_indicator.iter_mut() {
        *visibility = Visibility::Hidden;
    }
}
