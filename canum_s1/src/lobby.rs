use canum_play::prelude::*;
use canum_play::setup::lobby::get_marks;

pub(super) struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(setup::Fight("Gate".to_owned())), setup_lobby);
    }
}

fn setup_lobby(mut commands: Commands, save: Res<Save>) {
    use setup::cutscene::CutsceneDelete;
    commands.spawn((CutsceneDelete, Observer::new(handle_lobby_select)));
    commands.spawn((CutsceneDelete, Observer::new(quit_lobby_panel)));
    commands.spawn((CutsceneDelete, Observer::new(handle_lobby_shop)));
    if save
        .progress
        .has_boss_progress_and("Ant", |boss| boss.defeated)
    {
        commands.spawn((
            Transform::from_translation(Vec3::new(2000.0, 400.0, 0.0)),
            setup::shop::ShopIndicator,
        ));
    }
}

fn handle_lobby_shop(
    event: On<setup::lobby::LobbyShop>,
    mut commands: Commands,
    q_shop: Query<(), With<setup::shop::ShopIndicator>>,
) {
    if q_shop.iter().next().is_none() {
        return;
    }
    let index = (event.position.x / 800.0).floor() as i32;
    if index == 2 {
        commands.trigger(setup::shop::EnterShop {
            fight: "Shop_Ant".to_owned(),
        });
    }
}

#[allow(clippy::single_match)]
#[allow(clippy::too_many_arguments)]
fn handle_lobby_select(
    event: On<setup::lobby::LobbySelect>,
    mut commands: Commands,
    q_panel: Query<(), With<canum_ui::lobby::boss::BossPanel>>,
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
            commands.spawn((
                ChildOf(center),
                canum_ui::lobby::boss::boss_panel(
                    fonts,
                    canum_ui::lobby::boss::BossPanel {
                        name: lang.get("Apple_UiName").to_owned(),
                        fight_name: "Apple".to_owned(),
                        marks: get_marks(&save, "Apple"),
                        enter_color: Color::linear_rgb(1.0, 0.5, 0.5),
                    },
                    time,
                ),
            ));
        }
        1 => {
            commands.spawn((
                ChildOf(center),
                canum_ui::lobby::boss::boss_panel(
                    fonts,
                    canum_ui::lobby::boss::BossPanel {
                        name: lang.get("Turf_UiName").to_owned(),
                        fight_name: "Turf".to_owned(),
                        marks: get_marks(&save, "Turf"),
                        enter_color: Color::linear_rgb(0.5, 1.0, 0.5),
                    },
                    time,
                ),
            ));
        }
        2 => {
            commands.spawn((
                ChildOf(center),
                canum_ui::lobby::boss::boss_panel(
                    fonts,
                    canum_ui::lobby::boss::BossPanel {
                        name: lang.get("Ant_UiName").to_owned(),
                        fight_name: "Ant".to_owned(),
                        marks: get_marks(&save, "Ant"),
                        enter_color: Color::linear_rgb(0.2, 0.2, 0.2),
                    },
                    time,
                ),
            ));
        }
        3 => {
            commands.spawn((
                ChildOf(center),
                canum_ui::lobby::boss::boss_panel(
                    fonts,
                    canum_ui::lobby::boss::BossPanel {
                        name: lang.get("Runway_UiName").to_owned(),
                        fight_name: "Runway".to_owned(),
                        marks: get_marks(&save, "Runway"),
                        enter_color: Color::linear_rgb(0.5, 1.0, 0.5),
                    },
                    time,
                ),
            ));
        }
        4 => {
            commands.spawn((
                ChildOf(center),
                canum_ui::lobby::boss::boss_panel(
                    fonts,
                    canum_ui::lobby::boss::BossPanel {
                        name: lang.get("Wcat_UiName").to_owned(),
                        fight_name: "Wcat".to_owned(),
                        marks: get_marks(&save, "Wcat"),
                        enter_color: Color::linear_rgb(1.0, 1.0, 1.0),
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
    q_panel: Query<Entity, With<canum_ui::lobby::boss::BossPanel>>,
) {
    for entity in q_panel.iter() {
        commands.entity(entity).despawn();
    }
}
