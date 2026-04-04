use crate::prelude::*;

pub(super) struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(setup_ui);
    }
}

/// Marker for UI node.
#[derive(Component, Default)]
pub struct TopLeft;

/// Marker for UI node.
#[derive(Component, Default)]
pub struct BottomLeft;

fn setup_ui(
    event: On<canum_play::setup::PostStartSession>,
    save: Res<Save>,
    mut commands: Commands,
) {
    let root = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            canum_play::SessionOnly,
        ))
        .id();
    let top_left = commands
        .spawn((
            ChildOf(root),
            TopLeft,
            Node {
                position_type: PositionType::Absolute,
                left: px(10.0),
                top: px(10.0),
                ..default()
            },
        ))
        .id();
    commands.spawn((
        ChildOf(root),
        BottomLeft,
        Node {
            position_type: PositionType::Absolute,
            left: px(30.0),
            bottom: px(30.0),
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
