use super::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(WCAT_STATE.clone()), (spawn_wcat_and_dialogue,));
        app.world_mut()
            .register_component_hooks::<WcatDialogue>()
            .on_remove(|mut world, _context| {
                world.commands().trigger(WcatStart);
            });
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
    mut save: ResMut<Save>,
) {
    commands.spawn((WcatBoss,));
    commands.spawn((
        SessionOnly,
        canum_res::background::Background::new(CONFIG.display.screen_size),
        Animation::new("Wcat_Back", CONFIG.display.screen_size)
            .with_color(Color::default().with_alpha(0.6)),
    ));

    let Ok(mut player_transform) = q_player.single_mut() else {
        return;
    };
    player_transform.translation.x -= 200.0;

    if save.progress.first_time("Wcat_Before") {
        let Ok(bottom_center) = q_bottom_center.single() else {
            return;
        };
        commands.spawn((
            ChildOf(bottom_center),
            WcatDialogue,
            canum_ui::dialogue::Dialogue::from_config("Wcat_Before", lang),
        ));
    }
}
