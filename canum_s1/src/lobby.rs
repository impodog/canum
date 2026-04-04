use canum_play::prelude::*;

pub(super) struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(setup::Fight("Gate".to_owned())), update_observer);
    }
}

fn update_observer(mut commands: Commands) {
    commands.spawn((SessionOnly, Observer::new(handle_lobby_select)));
    commands.spawn((SessionOnly, Observer::new(quit_lobby_panel)));
}

#[allow(clippy::single_match)]
fn handle_lobby_select(
    event: On<setup::lobby::LobbySelect>,
    mut commands: Commands,
    q_panel: Query<(), With<canum_ui::boss::BossPanel>>,
    q_center: Query<Entity, With<canum_ui::Center>>,
    fonts: Res<canum_ui::Fonts>,
    lang: Res<Lang>,
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
                canum_ui::boss::boss_panel(
                    fonts,
                    canum_ui::boss::BossPanel {
                        name: lang.get("Apple_UiName").to_owned(),
                    },
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
