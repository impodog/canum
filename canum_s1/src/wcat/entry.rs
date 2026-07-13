use super::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(WCAT_STATE.clone()), (spawn_wcat_and_dialogue,));
    }
}

#[derive(Event, Default)]
pub struct WcatStart;

#[derive(Component, Default)]
struct WcatDialogue;

fn spawn_wcat_and_dialogue(
    mut commands: Commands,
    mut q_player: Query<&mut Transform, With<player::Player>>,
    q_bottom_center: Query<Entity, With<canum_ui::BottomCenter>>,
    lang: Res<Lang>,
) {
    commands.spawn((WcatBoss,));

    let Ok(mut player_transform) = q_player.single_mut() else {
        return;
    };
    player_transform.translation.x -= 200.0;

    let Ok(bottom_center) = q_bottom_center.single() else {
        return;
    };
    commands.spawn((
        ChildOf(bottom_center),
        WcatDialogue,
        canum_ui::dialogue::Dialogue::from_config("Wcat_Before", lang),
    ));
}
