use crate::prelude::*;

pub(super) struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(setup_ui);
    }
}

#[derive(Component, Default)]
pub struct TopLeft;

#[derive(Component, Default)]
pub struct BottomLeft;

#[derive(Component, Default)]
pub struct Center;

fn setup_ui(
    event: On<canum_play::setup::PostStartSession>,
    save: Res<Save>,
    mut commands: Commands,
) {
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: px(CONFIG.display.virtual_size.0),
                height: px(CONFIG.display.virtual_size.1),
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            UiAntiAlias::Off,
            canum_play::SessionOnly,
        ))
        .id();
    let top_left = commands
        .spawn((
            ChildOf(root),
            TopLeft,
            Node {
                position_type: PositionType::Absolute,
                left: px(3.0),
                top: px(3.0),
                ..default()
            },
        ))
        .id();
    commands.spawn((
        ChildOf(root),
        BottomLeft,
        Node {
            position_type: PositionType::Absolute,
            left: px(10.0),
            bottom: px(15.0),
            ..default()
        },
    ));
    commands.spawn((
        ChildOf(root),
        Center,
        Node {
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            margin: UiRect::all(Val::Auto),
            width: percent(80),
            height: percent(80),
            ..default()
        },
    ));
    match save.progress.selected_health.as_str() {
        "BasicHp" => {
            commands.spawn((
                ChildOf(top_left),
                crate::health::integer_health(event.health_entity, 6),
            ));
        }
        _ => {
            warn!("Unknown health type. No UI available.")
        }
    }
}
