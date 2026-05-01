use std::collections::{BTreeSet, VecDeque};

use super::*;

pub(super) struct CharmsPlugin;

impl Plugin for CharmsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, listen_charm_select_input);
        app.add_systems(FixedPostUpdate, update_charm_used);
        app.add_observer(update_charm_select);
    }
}

const CHARM_SELECT_MENU_LENGTH: usize = 9;
const CHARM_SELECT_MENU_SIZE: Vec2 = vec2(200.0, 300.0);

#[derive(Component, Default, Debug)]
struct CharmSelectMenu {
    charms: VecDeque<String>,
}

pub(super) fn charm_select_menu(kind: String) -> impl Bundle {
    (
        Node {
            align_content: AlignContent::Start,
            justify_content: JustifyContent::Start,
            margin: UiRect::all(Val::Auto),
            padding: UiRect::horizontal(px(10.0)),
            width: px(CHARM_SELECT_MENU_SIZE.x),
            height: px(CHARM_SELECT_MENU_SIZE.y),
            ..default()
        },
        CharmSelectMenu::default(),
        Animation::new(format!("{kind}_Equip_Charms"), CHARM_SELECT_MENU_SIZE),
    )
}

#[derive(Event, Clone, Copy)]
pub(super) enum CharmSelectInput {
    Next,
    NextPage,
    Prev,
    PrevPage,
    Update,
    Toggle,
}

fn listen_charm_select_input(
    key: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    q_menu: Query<(), With<CharmSelectMenu>>,
) {
    if q_menu.iter().next().is_none() {
        return;
    }
    if key.just_pressed(KeyCode::ArrowUp) {
        if key.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            commands.trigger(CharmSelectInput::PrevPage);
        } else {
            commands.trigger(CharmSelectInput::Prev);
        }
    }
    if key.just_pressed(KeyCode::ArrowDown) {
        if key.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            commands.trigger(CharmSelectInput::NextPage);
        } else {
            commands.trigger(CharmSelectInput::Next);
        }
    }
    if key.any_just_pressed([KeyCode::Enter, KeyCode::KeyC]) {
        commands.trigger(CharmSelectInput::Toggle);
    }
}

/// Returns an iterator over the next `n` elements after `start` in sorted order.
pub fn next_n_cyclic<'a, T>(
    set: &'a BTreeSet<T>,
    start: &'a T,
    n: usize,
) -> impl Iterator<Item = &'a T> + 'a
where
    T: Ord,
{
    set.range(start..)
        .skip(1)
        .chain(set.range(..=start))
        .take(n)
}

/// Returns an iterator over the previous `n` elements before `start` in reverse sorted order.
pub fn prev_n_cyclic<'a, T>(
    set: &'a BTreeSet<T>,
    start: &'a T,
    n: usize,
) -> impl Iterator<Item = &'a T> + 'a
where
    T: Ord,
{
    set.range(..start)
        .rev()
        .chain(set.range(start..).rev())
        .take(n)
}

fn update_charm_select(
    event: On<CharmSelectInput>,
    mut q_menu: Query<(Entity, &mut CharmSelectMenu)>,
    mut commands: Commands,
    save: Res<Save>,
) {
    let Ok((entity, mut menu)) = q_menu.single_mut() else {
        return;
    };
    if save.progress.gained_charms.is_empty() {
        // TODO: Maybe indicator for no charms?
        return;
    }
    if menu.charms.is_empty() {
        for charm in save
            .progress
            .gained_charms
            .iter()
            .take(CHARM_SELECT_MENU_LENGTH)
        {
            menu.charms.push_back(charm.to_owned());
        }
    }
    match *event {
        CharmSelectInput::Next => {
            let next_element = save
                .progress
                .gained_charms
                .range(menu.charms.back().unwrap().clone()..)
                .nth(1)
                .unwrap_or(save.progress.gained_charms.first().unwrap())
                .clone();
            menu.charms.pop_front();
            menu.charms.push_back(next_element);
        }
        CharmSelectInput::Prev => {
            let prev_element = save
                .progress
                .gained_charms
                .range(..menu.charms.back().unwrap().clone())
                .last()
                .unwrap_or(save.progress.gained_charms.last().unwrap())
                .clone();
            menu.charms.pop_back();
            menu.charms.push_front(prev_element);
        }
        CharmSelectInput::NextPage => {
            let next_elements = next_n_cyclic(
                &save.progress.gained_charms,
                menu.charms.back().unwrap(),
                CHARM_SELECT_MENU_LENGTH,
            )
            .cloned()
            .collect::<VecDeque<_>>();
            menu.charms = next_elements;
        }
        CharmSelectInput::PrevPage => {
            let prev_elements = prev_n_cyclic(
                &save.progress.gained_charms,
                menu.charms.front().unwrap(),
                CHARM_SELECT_MENU_LENGTH,
            )
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<VecDeque<_>>();
            menu.charms = prev_elements;
        }
        CharmSelectInput::Update => {}
        CharmSelectInput::Toggle => {
            let Some(selected) = menu.charms.front() else {
                return;
            };
            commands.trigger(UpdateCharms::Toggle(selected.clone()));
        }
    }

    commands.entity(entity).despawn_children();
    let mut position = vec2(20.0, 0.0);
    let mut children_charms = Vec::new();
    for charm in menu.charms.iter() {
        children_charms.push((
            Node {
                position_type: PositionType::Absolute,
                align_content: AlignContent::Center,
                left: px(position.x),
                top: px(position.y),
                width: px(32.0),
                height: px(32.0),
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            CharmImage(charm.clone()),
            Animation::new(format!("Charm_{charm}"), Vec2::new(32.0, 32.0)),
            ZIndex(1),
        ));
        position.y += 32.0;
    }
    commands.spawn((
        ChildOf(entity),
        Node {
            position_type: PositionType::Absolute,
            align_content: AlignContent::Start,
            justify_content: JustifyContent::Start,
            flex_direction: FlexDirection::Column,
            margin: UiRect::all(Val::Auto),
            left: px(-CHARM_SELECT_MENU_SIZE.x * 0.5),
            top: px(-CHARM_SELECT_MENU_SIZE.y * 0.5),
            ..default()
        },
        Children::spawn(children_charms),
    ));
}

#[derive(Component, Default)]
struct CharmImage(String);

fn update_charm_used(mut q_charm: Query<(&mut ImageNode, &CharmImage)>, save: Res<Save>) {
    q_charm.par_iter_mut().for_each(|(mut image_node, charm)| {
        if save.progress.charms.selected_charms.contains(&charm.0) {
            image_node.color.set_alpha(0.2);
        } else {
            image_node.color.set_alpha(1.0);
        }
    });
}
