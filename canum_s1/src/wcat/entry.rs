use canum_fx::wait_then_trigger;

use super::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(WCAT_STATE.clone()), (spawn_wcat,));
        app.world_mut()
            .register_component_hooks::<WcatDialogue>()
            .on_remove(|mut world, _context| {
                world.commands().trigger(WcatStart);
            });
        app.add_systems(OnEnter(WCAT_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(spawn_dialogue)));
            commands.spawn((SessionOnly, Observer::new(WcatStartTrigger::observer)));
        });
    }
}

#[derive(Event, Default)]
pub struct WcatStart;

wait_then_trigger!(WcatStartTrigger, WcatStart, 0.2);

#[derive(Component, Default)]
struct WcatDialogue;

fn spawn_wcat(mut commands: Commands, mut q_player: Query<&mut Transform, With<player::Player>>) {
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

    commands.spawn(WcatStartTrigger);
}

fn spawn_dialogue(
    _event: On<WcatStart>,
    mut commands: Commands,
    q_bottom_center: Query<Entity, With<canum_ui::BottomCenter>>,
    lang: Res<Lang>,
    mut save: ResMut<Save>,
) {
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
