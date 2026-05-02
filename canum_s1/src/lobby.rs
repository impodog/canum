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
    commands.spawn((setup::CutsceneDelete, Observer::new(handle_lobby_shop)));
}

fn handle_lobby_shop(event: On<setup::lobby::LobbyShop>, mut commands: Commands) {
    let index = (event.position.x / 800.0).floor() as i32;
    if index == 2 {
        commands.trigger(setup::StartSession {
            fight: "Shop_Ant".to_owned(),
        });
    }
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
    fn marks(save: &Save, name: &str) -> Vec<String> {
        save.progress
            .boss_progress
            .get(name)
            .map(|boss_progress| {
                boss_progress
                    .tasks
                    .iter()
                    .map(|task| format!("Mark_{task}"))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }
    if q_panel.single().is_ok() {
        return;
    }
    let Ok(center) = q_center.single() else {
        return;
    };
    let index = (event.position.x / 800.0).floor() as i32;
    match index {
        0 => {
            commands.spawn((
                ChildOf(center),
                canum_ui::boss::boss_panel(
                    fonts,
                    canum_ui::boss::BossPanel {
                        name: lang.get("Apple_UiName").to_owned(),
                        fight_name: "Apple".to_owned(),
                        marks: marks(&save, "Apple"),
                        enter_color: Color::linear_rgb(1.0, 0.5, 0.5),
                    },
                    time,
                ),
            ));
        }
        1 => {
            commands.spawn((
                ChildOf(center),
                canum_ui::boss::boss_panel(
                    fonts,
                    canum_ui::boss::BossPanel {
                        name: lang.get("Turf_UiName").to_owned(),
                        fight_name: "Turf".to_owned(),
                        marks: marks(&save, "Turf"),
                        enter_color: Color::linear_rgb(0.5, 1.0, 0.5),
                    },
                    time,
                ),
            ));
        }
        2 => {
            commands.spawn((
                ChildOf(center),
                canum_ui::boss::boss_panel(
                    fonts,
                    canum_ui::boss::BossPanel {
                        name: lang.get("Ant_UiName").to_owned(),
                        fight_name: "Ant".to_owned(),
                        marks: marks(&save, "Ant"),
                        enter_color: Color::linear_rgb(0.2, 0.2, 0.2),
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
