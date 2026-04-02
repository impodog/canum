use crate::prelude::*;

pub(super) struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(setup_ui);
    }
}

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
    let left_top = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(10.0),
                top: px(10.0),
                ..default()
            },
            ChildOf(root),
        ))
        .id();
    match save.progress.selected_health.as_str() {
        "BasicHp" => {
            commands.spawn((
                ChildOf(left_top),
                crate::health::integer_health(event.health_entity, 6),
            ));
        }
        _ => {
            warn!("Unknown health type. No UI available.")
        }
    }
}
