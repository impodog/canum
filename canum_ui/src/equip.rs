use crate::prelude::*;

pub(super) struct EquipPlugin;

impl Plugin for EquipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, listen_equip_keyboard_input);
        app.add_observer(setup_equip_menu);
    }
}

#[derive(Component, Default)]
#[require(Node)]
pub struct EquipMenu;

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

fn equip_menu(kind: String, level: EquipLevel) -> impl Bundle {
    (
        Node {
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Row,
            margin: UiRect::all(Val::Auto),
            ..default()
        },
        EquipMenu,
        canum_play::controls::OverrideMainControls,
        canum_play::SessionOnly,
        children![(
            Node {
                width: px(200.0),
                height: px(300.0),
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            Animation::new(format!("{kind}_Equip_{level}"), Vec2::new(100.0, 150.0))
        )],
    )
}

#[derive(Default, Event)]
struct CallEquipMenu;

fn listen_equip_keyboard_input(
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
) {
    if *state.get() == canum_play::setup::PlayState::Fighting {
        return;
    }
    if q_menu.iter().next().is_some() {
        return;
    }
    let kind = save.appearance.player.clone();
    let level = EquipLevel::S1;
    commands.spawn(equip_menu(kind, level));
}
