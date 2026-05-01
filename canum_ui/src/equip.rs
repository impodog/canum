use std::collections::{HashSet, VecDeque};

use crate::prelude::*;
use canum_save::*;

mod charms;
pub mod menu;
mod weapons;

pub(super) struct EquipPlugin;

impl Plugin for EquipPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            charms::CharmsPlugin,
            menu::MenuPlugin,
            weapons::WeaponsPlugin,
        ));
        app.init_resource::<CurrentMenuNumber>();
        app.add_systems(FixedUpdate, listen_equip_input);
        app.add_observer(setup_equip_menu)
            .add_observer(update_charms)
            .add_observer(update_weapons)
            .add_observer(change_select_menu);
    }
}

#[derive(Component, Default)]
#[require(Node)]
pub struct EquipMenu;

#[derive(Component, Default)]
#[require(Node)]
pub struct EquipStatusMenu;

#[derive(Component, Default)]
#[require(Node)]
pub struct CharmStatus;

#[derive(Component, Default)]
#[require(Node)]
pub struct WeaponStatus;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EquipLevel {
    /// Nothing is unlocked in S1.
    #[default]
    S1,
}
impl EquipLevel {
    pub const fn as_name(self) -> &'static str {
        match self {
            Self::S1 => "S1",
        }
    }
}
impl std::fmt::Display for EquipLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_name())
    }
}

const EQUIP_MENU_SIZE: Vec2 = vec2(200.0, 300.0);

fn equip_menu(kind: String, level: EquipLevel, fonts: &crate::Fonts) -> impl Bundle {
    (
        Node {
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Row,
            margin: UiRect::all(Val::Auto),
            padding: UiRect::horizontal(px(10)),
            ..default()
        },
        EquipMenu,
        canum_play::controls::OverrideMainControls,
        canum_play::SessionOnly,
        children![
            (
                Node {
                    width: px(EQUIP_MENU_SIZE.x),
                    height: px(EQUIP_MENU_SIZE.y),
                    margin: UiRect::all(Val::Auto),
                    ..default()
                },
                EquipStatusMenu,
                Animation::new(format!("{kind}_Equip_{level}"), EQUIP_MENU_SIZE),
                children![
                    (
                        Node {
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        CharmStatus,
                    ),
                    (
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(50),
                            ..default()
                        },
                        WeaponStatus,
                    )
                ]
            ),
            (
                weapons::WeaponSelectMenu,
                menu::select_menu(kind.clone(), "Weapon".to_owned(), fonts)
            )
        ],
    )
}

#[derive(Default, Event)]
struct CallEquipMenu;

fn listen_equip_input(
    key: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    q_menu: Query<Entity, With<EquipMenu>>,
) {
    if key.just_pressed(KeyCode::KeyE) {
        commands.trigger(CallEquipMenu);
    }
    if key.any_just_pressed([KeyCode::Escape, KeyCode::Backspace]) {
        for entity in q_menu.iter() {
            commands.entity(entity).despawn();
        }
    }
    if key.just_pressed(KeyCode::ArrowRight) {
        commands.trigger(ShiftMenuNumber(1));
    }
    if key.just_pressed(KeyCode::ArrowLeft) {
        commands.trigger(ShiftMenuNumber(-1));
    }
}

fn setup_equip_menu(
    _event: On<CallEquipMenu>,
    state: Res<State<canum_play::setup::PlayState>>,
    q_menu: Query<(), With<EquipMenu>>,
    mut commands: Commands,
    save: Res<Save>,
    fonts: Res<crate::Fonts>,
) {
    if *state.get() == canum_play::setup::PlayState::Fighting {
        return;
    }
    if q_menu.iter().next().is_some() {
        return;
    }
    let kind = save.appearance.player.clone();
    let level = EquipLevel::S1;
    commands.spawn(equip_menu(kind, level, &fonts));
    commands.trigger(UpdateCharms::NoAction);
    commands.trigger(UpdateWeapons::NoAction);
    commands.trigger(menu::SelectInput::Update);
}

fn show_charms(charms: &Charms) -> Option<impl Bundle> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Slot {
        None,
        Defensive,
        Offensive,
        General,
    }
    fn find_first_slot(charm: &str, slots: &[Slot], slot_occupy: &mut [Option<String>]) -> bool {
        let Some(details) = CONFIG.values.charm.get(charm) else {
            return false;
        };

        // info!(
        //     "Cost of {charm} is ({}, {}, {})",
        //     details.cost.defensive(),
        //     details.cost.offensive(),
        //     details.cost.general()
        // );

        if details.cost.general() {
            let mut ok = false;
            for index in 0..slots.len() {
                if slot_occupy[index].is_none() && slots[index] == Slot::General {
                    slot_occupy[index] = Some(charm.to_owned());
                    ok = true;
                    break;
                }
            }
            if !ok {
                return false;
            }
        }
        if details.cost.defensive() {
            // Prioritize corresponding slots instead of general slot. Same in offensive.
            let mut ok = false;
            for index in 0..slots.len() {
                if slot_occupy[index].is_none() && slots[index] == Slot::Defensive {
                    slot_occupy[index] = Some(charm.to_owned());
                    ok = true;
                    break;
                }
            }
            if !ok {
                for index in 0..slots.len() {
                    if slot_occupy[index].is_none() && slots[index] == Slot::General {
                        slot_occupy[index] = Some(charm.to_owned());
                        ok = true;
                        break;
                    }
                }
            }
            if !ok {
                return false;
            }
        }
        if details.cost.offensive() {
            let mut ok = false;
            for index in 0..slots.len() {
                if slot_occupy[index].is_none() && slots[index] == Slot::Offensive {
                    slot_occupy[index] = Some(charm.to_owned());
                    ok = true;
                    break;
                }
            }
            if !ok {
                for index in 0..slots.len() {
                    if slot_occupy[index].is_none() && slots[index] == Slot::General {
                        slot_occupy[index] = Some(charm.to_owned());
                        ok = true;
                        break;
                    }
                }
            }
            if !ok {
                return false;
            }
        }
        true
    }

    let mut slots = Vec::new();
    slots.push(if charms.has_defensive {
        Slot::Defensive
    } else {
        Slot::None
    });
    slots.push(Slot::General);
    slots.push(if charms.has_defensive {
        Slot::Offensive
    } else {
        Slot::None
    });
    let mut slot_occupy = Vec::<Option<String>>::new();
    slot_occupy.resize(slots.len(), None);

    for charm in charms.selected_charms.iter() {
        if !find_first_slot(charm, &slots, &mut slot_occupy) {
            return None;
        }
    }

    let mut position = vec2(50.0, 50.0);
    const SIZE: Vec2 = vec2(32.0, 32.0);
    let mut images = Vec::new();
    for charm in slot_occupy.iter() {
        if let Some(charm) = charm {
            images.push((
                Node {
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    margin: UiRect::all(Val::Auto),
                    left: px(position.x - SIZE.x * 0.5),
                    top: px(position.y - SIZE.y * 0.5),
                    width: px(SIZE.x),
                    height: px(SIZE.y),
                    ..default()
                },
                Animation::new(format!("Charm_{charm}"), SIZE),
            ));
        }
        position.x += 50.0;
    }
    Some((
        Node {
            position_type: PositionType::Absolute,
            margin: UiRect::all(Val::Auto),
            left: px(-EQUIP_MENU_SIZE.x * 0.5),
            top: px(-EQUIP_MENU_SIZE.y * 0.5),
            ..default()
        },
        ZIndex(1),
        Children::spawn(images),
    ))
}

/// When spawning menu or changing charm equip, update all charm positions.
#[derive(Default, Event)]
enum UpdateCharms {
    #[default]
    NoAction,
    Toggle(String),
}

fn update_charms(
    event: On<UpdateCharms>,
    q_menu: Query<Entity, With<CharmStatus>>,
    mut commands: Commands,
    mut save: ResMut<Save>,
) {
    let Ok(menu) = q_menu.single() else {
        return;
    };
    let new_charms = save
        .progress
        .charms
        .selected_charms
        .iter()
        .filter(|charm| save.progress.gained_charms.contains(*charm))
        .cloned()
        .collect::<HashSet<_>>();
    let mut new_charms = Charms {
        selected_charms: new_charms,
        ..save.progress.charms
    };
    let is_equip = match *event {
        UpdateCharms::NoAction => None,
        UpdateCharms::Toggle(ref charm) => {
            if !new_charms.selected_charms.remove(charm) {
                new_charms.selected_charms.insert(charm.clone());
                Some(true)
            } else {
                Some(false)
            }
        }
    };
    // info!("New charms are :{new_charms:?}");
    if let Some(charms) = show_charms(&new_charms) {
        // info!("CHARMS OK!");
        commands.entity(menu).insert(charms);
        save.progress.selected_effects.clear();
        for charm in new_charms.selected_charms.iter() {
            let Some(details) = CONFIG.values.charm.get(charm) else {
                continue;
            };
            save.progress
                .selected_effects
                .extend(details.effects.iter().cloned());
        }
        save.progress.charms = new_charms;
        if let Some(is_equip) = is_equip {
            if is_equip {
                commands.spawn(canum_res::sound::Sound::new("Ui_Equip"));
            } else {
                commands.spawn(canum_res::sound::Sound::new("Ui_Unequip"));
            }
        }
    } else {
        commands.spawn(canum_res::sound::Sound::new("Ui_EquipError"));
    }
}

/// When spawning menu or changing weapon equip, update all weapon positions. Same as charms.
#[derive(Default, Event)]
enum UpdateWeapons {
    #[default]
    NoAction,
    Toggle(String),
}

fn update_weapons(
    event: On<UpdateWeapons>,
    mut commands: Commands,
    q_menu: Query<Entity, With<WeaponStatus>>,
    mut save: ResMut<Save>,
) {
    let Ok(menu) = q_menu.single() else {
        return;
    };
    let mut new_weapons = save
        .progress
        .selected_weapons
        .iter()
        .filter(|weapon| save.progress.gained_weapons.contains(*weapon))
        .cloned()
        .collect::<VecDeque<_>>();
    match *event {
        UpdateWeapons::NoAction => {}
        UpdateWeapons::Toggle(ref weapon) => {
            if let Some((index, _)) = new_weapons
                .iter()
                .enumerate()
                .find(|(_, element)| **element == *weapon)
            {
                new_weapons.remove(index);
            } else {
                new_weapons.push_back(weapon.clone());
            }
            commands.spawn(canum_res::sound::Sound::new("Ui_Equip"));
        }
    }
    // New weapons kick older weapons.
    while new_weapons.len() > save.progress.weapon_slots {
        new_weapons.pop_front();
    }
    commands.entity(menu).despawn_children();
    let mut position = vec2(100.0, 50.0);
    const SIZE: Vec2 = vec2(32.0, 32.0);
    for weapon in new_weapons.iter() {
        commands.spawn((
            ChildOf(menu),
            Node {
                width: px(SIZE.x),
                height: px(SIZE.y),
                left: px(position.x - SIZE.x * 0.5),
                top: px(position.y - SIZE.y * 0.5),
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            Animation::new(format!("Weapon_{weapon}"), SIZE),
        ));
        position.y += 50.0;
    }
    save.progress.selected_weapons = new_weapons.into();
}

/// Stores the current menu that the player is able to interact with.
/// 0 - Weapons; 1 - Charms
#[derive(Resource, Deref, DerefMut, Default)]
struct CurrentMenuNumber(i8);
const TOTAL_MENUS: i8 = 2;

#[derive(Event, Deref, DerefMut)]
struct ShiftMenuNumber(i8);

fn change_select_menu(
    event: On<ShiftMenuNumber>,
    mut commands: Commands,
    mut q_menu: Query<&mut menu::SelectMenu>,
    q_node: Query<Entity, With<menu::SelectMenuNode>>,
    mut current_number: ResMut<CurrentMenuNumber>,
) {
    let Ok(mut select_menu) = q_menu.single_mut() else {
        return;
    };
    let Ok(entity) = q_node.single() else {
        return;
    };
    **current_number = (**current_number + **event + TOTAL_MENUS) % TOTAL_MENUS;
    commands
        .entity(entity)
        .try_remove::<charms::CharmSelectMenu>()
        .try_remove::<weapons::WeaponSelectMenu>();
    match **current_number {
        0 => {
            commands.entity(entity).insert(weapons::WeaponSelectMenu);
            select_menu.group = "Weapon".to_owned();
        }
        1 => {
            commands.entity(entity).insert(charms::CharmSelectMenu);
            select_menu.group = "Charm".to_owned();
        }
        _ => {
            unreachable!("Should be covered under TOTAL_MENUS");
        }
    }
    select_menu.options.clear();
    commands.trigger(menu::UpdateMenuStyle);
    commands.trigger(menu::SelectInput::Update);
}
