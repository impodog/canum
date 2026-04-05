//! Creates and manages pre-fight boss information panel.

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
pub struct BossPanel {
    pub name: String,
    pub fight_name: String,
    pub marks: Vec<String>,
}

fn boss_panel_marks(marks: &[String]) -> impl Bundle + use<> {
    let mut images = Vec::new();
    for mark in marks.iter() {
        images.push((
            Node { ..default() },
            Animation::new(mark, Vec2::new(32.0, 32.0)).with_repeating(),
        ));
    }
    (
        Node {
            padding: UiRect::all(px(5.0)),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        Children::spawn(images),
    )
}

pub fn boss_panel(fonts: impl AsRef<Fonts>, panel: BossPanel) -> impl Bundle {
    let fonts = fonts.as_ref();
    let title = (
        Node {
            justify_content: JustifyContent::Center,
            margin: UiRect::all(Val::Auto),
            ..default()
        },
        Text::new(&panel.name),
        TextFont {
            font: fonts.title.clone(),
            font_size: 50.0,
            ..default()
        },
    );
    let marks = boss_panel_marks(&panel.marks);
    // let start_button = (Node { ..default() }, Button);
    (
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            margin: UiRect::all(Val::Auto),
            padding: UiRect::all(px(10.0)),
            width: percent(100),
            height: percent(100),
            ..default()
        },
        panel,
        BackgroundColor(Color::linear_rgba(0.1, 0.1, 0.1, 0.8)),
        BoxShadow::default(),
        children![title, marks],
    )
}

fn handle_panel_select(
    _event: On<setup::lobby::LobbySelect>,
    mut commands: Commands,
    q_panel: Query<(Entity, Ref<BossPanel>)>,
    mut next_state: ResMut<NextState<setup::GameState>>,
    state: Res<State<setup::GameState>>,
    q_camera: Query<Entity, With<canum_res::camera::PixelCamera>>,
) {
    if *state.get() == setup::GameState::Cutscene {
        return;
    }
    let Ok(camera_entity) = q_camera.single() else {
        return;
    };
    let Ok((panel_entity, panel)) = q_panel.single() else {
        return;
    };
    // Disallow spawning and entering the panel on the same frame.
    if panel.is_added() {
        return;
    }
    let sound_entity = commands
        .spawn((setup::CutsceneWait, canum_res::sound::Sound::new("Confirm")))
        .id();
    commands
        .spawn((
            ChildOf(camera_entity),
            canum_fx::transition::PureColor {
                destroy: sound_entity,
                duration: std::time::Duration::from_secs_f32(2.0),
                color: Color::linear_rgb(0.6, 0.2, 0.2),
                remove_self: true,
            },
            children![setup::CutsceneWait],
        ))
        .observe(unleash_cutscene_when_pure_color_half_point);
    commands.insert_resource(setup::CutsceneNext {
        event: setup::StartSession {
            fight: panel.fight_name.clone(),
        },
    });
    commands.entity(panel_entity).despawn();
    next_state.set(setup::GameState::Cutscene);
}

fn unleash_cutscene_when_pure_color_half_point(
    event: On<canum_fx::transition::PureColorHalfPoint>,
    mut commands: Commands,
) {
    commands.entity(event.entity).despawn();
}
