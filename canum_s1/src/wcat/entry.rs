use canum_fx::wait_then_trigger;

use super::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(WCAT_STATE.clone()), (spawn_wcat,));
        app.add_systems(OnEnter(WCAT_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(spawn_dialogue)));
            commands.spawn((SessionOnly, Observer::new(on_dialogue_complete)));
            commands.spawn((SessionOnly, Observer::new(spawn_music)));
        });
    }
}

#[derive(Event, Default)]
pub struct WcatStart;
wait_then_trigger!(WcatStartTrigger, WcatStart, 0.2);

#[derive(Event, Default)]
pub struct WcatFightStart;
wait_then_trigger!(WcatFightStartTrigger, WcatFightStart, 1.0);

#[derive(Component, Default)]
struct WcatDialogue;

fn spawn_wcat(mut commands: Commands, mut q_player: Query<&mut Transform, With<player::Player>>) {
    let wcat = commands.spawn((WcatBoss,)).id();
    commands.spawn((
        ChildOf(wcat),
        enemy::health::EnemySensor,
        Collider::rectangle(50.0, 30.0),
    ));
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

    commands
        .spawn(WcatStartTrigger)
        .observe(WcatStartTrigger::observer);
}

fn spawn_dialogue(
    _event: On<WcatStart>,
    mut commands: Commands,
    q_bottom_center: Query<Entity, With<canum_ui::BottomCenter>>,
    lang: Res<Lang>,
    mut save: ResMut<Save>,
) {
    if save.progress.first_time("Wcat_Prefight") {
        let Ok(bottom_center) = q_bottom_center.single() else {
            return;
        };
        commands.spawn((
            ChildOf(bottom_center),
            WcatDialogue,
            canum_ui::dialogue::Dialogue::from_config("Wcat_Prefight", lang),
        ));
    } else {
        // Simulate post-dialogue event.
        commands.trigger(canum_ui::dialogue::DialogueComplete);
    }
}

fn on_dialogue_complete(
    _event: On<canum_ui::dialogue::DialogueComplete>,
    mut commands: Commands,
    q_bottom_left: Query<Entity, With<canum_ui::BottomLeft>>,
    fonts: Res<canum_ui::Fonts>,
    lang: Res<Lang>,
) {
    let Ok(bottom_left) = q_bottom_left.single() else {
        return;
    };
    commands.spawn((
        ChildOf(bottom_left),
        canum_ui::text::popup_title(
            fonts.title.clone(),
            lang.get("Wcat_BossTitle"),
            Duration::from_secs_f32(1.0),
        ),
    ));
    commands
        .spawn(WcatFightStartTrigger)
        .observe(WcatFightStartTrigger::observer);
}

fn spawn_music(_event: On<WcatFightStart>, mut commands: Commands) {
    commands.spawn((Music, Sound::new("Wcat_Bgm")));
}
