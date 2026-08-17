use std::collections::{BTreeSet, VecDeque};

use canum_save::Lang;

use crate::prelude::*;

pub(super) struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (listen_select_input_keyboard, listen_select_input_gamepad),
        );
        app.add_systems(FixedPostUpdate, update_option_info);
        app.add_observer(update_options_display)
            .add_observer(update_menu_style)
            .add_observer(play_select_input_sound);
    }
}

/// Returns the next element in the set.
pub fn next_cyclic<'a, T>(set: &'a BTreeSet<T>, start: &'a T) -> &'a T
where
    T: Ord,
{
    set.range(start..).nth(1).unwrap_or(set.first().unwrap())
}

/// Returns the previous element in the set.
pub fn prev_cyclic<'a, T>(set: &'a BTreeSet<T>, start: &'a T) -> &'a T
where
    T: Ord,
{
    set.range(..start).last().unwrap_or(set.last().unwrap())
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

pub const SELECT_MENU_LENGTH: usize = 9;
pub const SELECT_MENU_SIZE: Vec2 = vec2(200.0, 300.0);

#[derive(Component, Default)]
#[require(Node)]
pub struct SelectMenuNode;

#[derive(Component, Default, Debug)]
pub struct SelectMenu {
    pub options: VecDeque<String>,
    pub group: String,
}

#[derive(Component, Default, Debug)]
pub struct InfoMenu {
    pub previous: String,
}

pub fn select_menu(kind: String, group: String, fonts: &crate::Fonts) -> impl Bundle {
    (
        Node {
            align_content: AlignContent::Start,
            justify_content: JustifyContent::Start,
            margin: UiRect::all(Val::Auto),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        SelectMenuNode,
        children![
            (
                Node {
                    margin: UiRect::all(Val::Auto),
                    width: px(SELECT_MENU_SIZE.x),
                    height: px(SELECT_MENU_SIZE.y),
                    ..default()
                },
                SelectMenu {
                    group: group.clone(),
                    ..default()
                },
                Animation::new(format!("{kind}_Equip_{group}_Select"), SELECT_MENU_SIZE),
            ),
            (
                Node {
                    margin: UiRect::all(Val::Auto),
                    width: px(SELECT_MENU_SIZE.x),
                    height: px(SELECT_MENU_SIZE.y),
                    ..default()
                },
                Animation::new(format!("{kind}_Equip_{group}_Desc"), SELECT_MENU_SIZE),
                children![(
                    Node {
                        align_content: AlignContent::Start,
                        justify_content: JustifyContent::Start,
                        position_type: PositionType::Absolute,
                        margin: UiRect::all(Val::Auto),
                        ..default()
                    },
                    InfoMenu::default(),
                    Text::new(""),
                    TextLayout {
                        justify: Justify::Left,
                        linebreak: LineBreak::WordBoundary
                    },
                    TextFont {
                        font: fonts.desc.clone().into(),
                        font_size: FontSize::Px(25.0),
                        font_smoothing: FontSmoothing::None,
                        ..default()
                    },
                    TextColor::WHITE,
                    children![(
                        TextSpan::new(""),
                        TextFont {
                            font: fonts.desc.clone().into(),
                            font_size: FontSize::Px(16.0),
                            font_smoothing: FontSmoothing::None,
                            ..default()
                        },
                        TextColor::WHITE,
                    )]
                )]
            )
        ],
    )
}

/// When the menu is changed but reused, update background image styles.
#[derive(Event)]
pub struct UpdateMenuStyle;

fn update_menu_style(
    _event: On<UpdateMenuStyle>,
    q_menu: Query<&Children, With<SelectMenuNode>>,
    q_select_menu: Query<&SelectMenu>,
    mut q_animation: Query<&mut Animation>,
) {
    let Ok(children) = q_menu.single() else {
        return;
    };
    let Ok(select_menu) = q_select_menu.single() else {
        return;
    };
    for child in children.iter() {
        let Ok(mut animation) = q_animation.get_mut(child) else {
            continue;
        };
        if let Some(right) = animation.name.rfind('_')
            && let Some(left) = animation.name[0..right].rfind('_')
        {
            animation
                .name
                .replace_range(left + 1..right, &select_menu.group);
        }
    }
}

/// Specific menus only need to respond to this.
#[derive(Event, Clone, Copy)]
pub enum SelectInput {
    Next,
    NextPage,
    Prev,
    PrevPage,
    Update,
    Toggle,
}

fn listen_select_input_keyboard(
    key: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    q_menu: Query<(), With<SelectMenu>>,
    save: Res<Save>,
) {
    if q_menu.iter().next().is_none() {
        return;
    }
    if key.just_pressed(save.keyboard.move_up) {
        if key.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            commands.trigger(SelectInput::PrevPage);
        } else {
            commands.trigger(SelectInput::Prev);
        }
    }
    if key.just_pressed(save.keyboard.move_down) {
        if key.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            commands.trigger(SelectInput::NextPage);
        } else {
            commands.trigger(SelectInput::Next);
        }
    }
    if key.any_just_pressed([save.keyboard.confirm, save.keyboard.primary_attack]) {
        commands.trigger(SelectInput::Toggle);
    }
}

fn listen_select_input_gamepad(
    gamepad: Single<&Gamepad, With<canum_play::controls::MainGamepad>>,
    mut commands: Commands,
    q_menu: Query<(), With<SelectMenu>>,
    save: Res<Save>,
) {
    if q_menu.iter().next().is_none() {
        return;
    }
    if gamepad.just_pressed(GamepadButton::DPadUp) {
        if gamepad.pressed(save.gamepad.dash) {
            commands.trigger(SelectInput::PrevPage);
        } else {
            commands.trigger(SelectInput::Prev);
        }
    }
    if gamepad.just_pressed(GamepadButton::DPadDown) {
        if gamepad.pressed(save.gamepad.dash) {
            commands.trigger(SelectInput::NextPage);
        } else {
            commands.trigger(SelectInput::Next);
        }
    }
    if gamepad.just_pressed(save.gamepad.confirm) {
        commands.trigger(SelectInput::Toggle);
    }
}

fn play_select_input_sound(event: On<SelectInput>, mut commands: Commands) {
    match *event {
        SelectInput::Next | SelectInput::Prev => {
            commands.spawn(canum_res::sound::Sound::new("Ui_Navigate"));
        }
        SelectInput::NextPage | SelectInput::PrevPage => {
            commands.spawn(canum_res::sound::Sound::new("Ui_Navigate").with_volume_add(5.0));
        }
        _ => {}
    }
}

#[derive(EntityEvent)]
pub struct UpdateOptionsDisplay {
    pub entity: Entity,
}

#[derive(Default, Debug, Component)]
pub struct OptionName(pub String);

fn update_options_display(
    event: On<UpdateOptionsDisplay>,
    mut commands: Commands,
    q_menu: Query<&SelectMenu>,
    fonts: Res<crate::Fonts>,
    lang: Res<Lang>,
) {
    let Ok(menu) = q_menu.get(event.entity) else {
        return;
    };
    commands.entity(event.entity).despawn_children();
    let mut position = vec2(20.0, 0.0);
    let parent = commands
        .spawn((
            ChildOf(event.entity),
            Node {
                position_type: PositionType::Absolute,
                align_content: AlignContent::Start,
                justify_content: JustifyContent::Start,
                flex_direction: FlexDirection::Column,
                margin: UiRect::all(Val::Auto),
                left: px(-SELECT_MENU_SIZE.x * 0.5),
                top: px(-SELECT_MENU_SIZE.y * 0.5),
                ..default()
            },
        ))
        .id();
    for option in menu.options.iter() {
        let option_name = format!("{}_{option}", menu.group);
        commands.spawn((
            ChildOf(parent),
            Node {
                position_type: PositionType::Absolute,
                align_content: AlignContent::Center,
                left: px(position.x),
                top: px(position.y),
                ..default()
            },
            children![
                (
                    Node {
                        align_content: AlignContent::Center,
                        margin: UiRect::all(Val::Auto),
                        width: px(32.0),
                        height: px(32.0),
                        ..default()
                    },
                    OptionName(option_name.clone()),
                    Animation::new(option_name, Vec2::new(32.0, 32.0)),
                ),
                (
                    Node {
                        align_content: AlignContent::Center,
                        margin: UiRect::all(Val::Auto),
                        ..default()
                    },
                    Text::new(lang.get(&format!("{}_{option}_Short", menu.group))),
                    TextLayout {
                        justify: Justify::Left,
                        linebreak: LineBreak::NoWrap
                    },
                    TextFont {
                        font: fonts.desc.clone().into(),
                        font_size: FontSize::Px(20.0),
                        font_smoothing: FontSmoothing::None,
                        ..default()
                    },
                    TextColor::WHITE,
                )
            ],
            ZIndex(1),
        ));
        position.y += 32.0;
    }
}

fn update_option_info(
    mut q_info: Query<(&mut InfoMenu, &mut Text, &Children)>,
    q_select_menu: Query<&SelectMenu>,
    mut q_text_span: Query<&mut TextSpan>,
    lang: Res<Lang>,
) {
    let Ok(select_menu) = q_select_menu.single() else {
        return;
    };
    for (mut menu, mut text, children) in q_info.iter_mut() {
        if let Some(current) = select_menu.options.front()
            && menu.previous != *current
        {
            menu.previous = current.clone();
            text.0 = lang
                .get(&format!("{}_{current}_Title", select_menu.group))
                .to_owned()
                + "\n";
            let Some(child) = children.iter().next() else {
                continue;
            };
            let Ok(mut text_span) = q_text_span.get_mut(child) else {
                continue;
            };
            text_span.0 = lang
                .get(&format!("{}_{current}_Desc", select_menu.group))
                .to_owned();
        }
    }
}
