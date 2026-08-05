use canum_play::prelude::*;
use canum_play::setup::lobby::get_marks;

pub(super) struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(setup::Fight("House".to_owned())), setup_lobby);
    }
}

fn setup_lobby(mut commands: Commands) {
    use setup::cutscene::CutsceneDelete;
    commands.spawn((CutsceneDelete, Observer::new(handle_lobby_select)));
    commands.spawn((CutsceneDelete, Observer::new(quit_lobby_panel)));
}

#[allow(clippy::single_match)]
#[allow(clippy::too_many_arguments)]
fn handle_lobby_select(
    event: On<setup::lobby::LobbySelect>,
    mut commands: Commands,
    q_panel: Query<(), With<canum_ui::lobby::boss::BossPanel>>,
    q_center: Query<Entity, With<canum_ui::Center>>,
    _q_camera: Query<Entity, With<canum_res::camera::PixelCamera>>,
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
                        name: lang.get("Laser_UiName").to_owned(),
                        fight_name: "Laser".to_owned(),
                        marks: get_marks(&save, "Laser"),
                        enter_color: Color::linear_rgb(0.8, 0.05, 0.05),
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
