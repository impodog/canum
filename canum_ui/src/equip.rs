use std::collections::HashSet;

use crate::prelude::*;
use canum_save::*;

mod charms;

pub(super) struct EquipPlugin;

impl Plugin for EquipPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((charms::CharmsPlugin,));
        app.add_systems(FixedUpdate, listen_equip_input);
        app.add_observer(setup_equip_menu)
            .add_observer(update_charms);
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
                children![(
                    Node {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    CharmStatus,
                )]
            ),
            charms::charm_select_menu(kind.clone(), fonts)
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
    commands.trigger(charms::CharmSelectInput::Update);
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
