//! Creates and manages pre-fight boss information panel.

use std::time::Duration;

use crate::prelude::*;
use crate::text::Fonts;
use canum_play::setup;

pub(super) struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_panel_select);
    }
}

/// Marks and stores constants of a boss panel.
/// Other variables are in separate components.
#[derive(Component, Debug)]
#[require(BossPanelAdded)]
pub struct BossPanel {
    pub name: String,
    pub fight_name: String,
    pub marks: Vec<String>,
    pub enter_color: Color,
}
#[derive(Component, Default)]
struct BossPanelAdded(Duration);

fn boss_panel_marks(marks: &[String]) -> impl Bundle + use<> {
    let mut images = Vec::new();
    for mark in marks.iter() {
        images.push((
            Node {
                width: px(40.0),
                height: px(40.0),
                margin: UiRect::horizontal(Val::Auto),
                ..default()
            },
            Animation::new(mark, Vec2::new(40.0, 40.0)).with_repeating(),
        ));
    }
    (
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            margin: UiRect::horizontal(Val::Auto),
            ..default()
        },
        Children::spawn(images),
    )
}

pub fn boss_panel(fonts: impl AsRef<Fonts>, panel: BossPanel, time: Res<Time>) -> impl Bundle {
    let fonts = fonts.as_ref();
    let title = (
        Node {
            justify_content: JustifyContent::Center,
            margin: UiRect::horizontal(Val::Auto),
            ..default()
        },
        Text::new(&panel.name),
        TextFont {
            font: fonts.title.clone(),
            font_size: 50.0,
            font_smoothing: FontSmoothing::None,
            ..default()
        },
    );
    let marks = boss_panel_marks(&panel.marks);
    // let start_button = (Node { ..default() }, Button);
    (
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            margin: UiRect::horizontal(Val::Auto),
            padding: UiRect::vertical(px(10.0)),
            width: percent(100),
            height: percent(100),
            ..default()
        },
        panel,
        BossPanelAdded(time.elapsed()),
        BackgroundColor(Color::linear_rgba(0.1, 0.1, 0.1, 0.8)),
        BoxShadow::default(),
        children![title, marks],
    )
}

fn handle_panel_select(
    _event: On<setup::lobby::LobbySelect>,
    mut commands: Commands,
    q_panel: Query<(&BossPanel, &BossPanelAdded)>,
    state: Res<State<setup::GameState>>,
    q_camera: Query<Entity, With<canum_res::camera::PixelCamera>>,
    time: Res<Time>,
) {
    if *state.get() == setup::GameState::Cutscene {
        return;
    }
    let Ok(camera_entity) = q_camera.single() else {
        return;
    };
    let Ok((panel, panel_add_time)) = q_panel.single() else {
        return;
    };
    // Disallow spawning and entering the panel on the same frame.
    if time.elapsed() - panel_add_time.0 <= Duration::from_millis(100) {
        return;
    }

    let sound_entity = commands
        .spawn((
            setup::cutscene::CutsceneWait,
            canum_res::sound::Sound::new("Confirm"),
        ))
        .id();
    commands.spawn((
        ChildOf(camera_entity),
        canum_play::setup::cutscene::PureColorCutscene {
            transition: canum_fx::transition::PureColor {
                destroy: Some(sound_entity),
                duration: std::time::Duration::from_secs_f32(1.5),
                color: panel.enter_color,
                remove_self: true,
            },
            fight: panel.fight_name.clone(),
        },
    ));
}
