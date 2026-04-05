use canum_play::prelude::*;

pub(super) struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(setup::Fight("Gate".to_owned())), update_observer);
    }
}

fn update_observer(mut commands: Commands) {
    commands.spawn((setup::CutsceneDelete, Observer::new(handle_lobby_select)));
    commands.spawn((setup::CutsceneDelete, Observer::new(quit_lobby_panel)));
}

#[allow(clippy::single_match)]
#[allow(clippy::too_many_arguments)]
fn handle_lobby_select(
    event: On<setup::lobby::LobbySelect>,
    mut commands: Commands,
    q_panel: Query<(), With<canum_ui::boss::BossPanel>>,
    q_center: Query<Entity, With<canum_ui::Center>>,
    fonts: Res<canum_ui::Fonts>,
    lang: Res<Lang>,
    save: Res<Save>,
    time: Res<Time>,
) {
    if q_panel.single().is_ok() {
        return;
    }
    let Ok(center) = q_center.single() else {
        return;
    };
    let index = (event.position.x / 800.0).floor() as i32;
    match index {
        0 => {
            let marks = save
                .progress
                .boss_progress
                .get("Apple")
                .map(|boss_progress| {
                    boss_progress
                        .tasks
                        .iter()
                        .map(|task| format!("Mark_{task}"))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            commands.spawn((
                ChildOf(center),
                SessionOnly,
                canum_ui::boss::boss_panel(
                    fonts,
                    canum_ui::boss::BossPanel {
                        name: lang.get("Apple_UiName").to_owned(),
                        fight_name: "Apple".to_owned(),
                        marks,
                    },
                    time,
                ),
            ));
        }
        _ => {}
    }
}

fn quit_lobby_panel(
    _event: On<setup::lobby::LobbyQuit>,
    mut commands: Commands,
    q_panel: Query<Entity, With<canum_ui::boss::BossPanel>>,
) {
    for entity in q_panel.iter() {
        commands.entity(entity).despawn();
    }
}
