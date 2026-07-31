use crate::prelude::*;

pub(super) struct FailurePlugin;

impl Plugin for FailurePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(update_player_status)
            .add_observer(update_save);
    }
}

/// Global event that the player died.
#[derive(Event)]
pub struct PlayerFail;

fn update_player_status(
    _event: On<PlayerFail>,
    mut commands: Commands,
    q_player: Query<Entity, With<crate::player::Player>>,
    mut next_state: ResMut<NextState<crate::setup::GameState>>,
    state: Res<State<crate::setup::GameState>>,
) {
    if *state.get() == crate::setup::GameState::Cutscene {
        return;
    }
    // Prevents player control.
    commands.remove_resource::<crate::player::PrimaryPlayer>();
    for entity in q_player.iter() {
        commands
            .entity(entity)
            .remove::<Collider>()
            .remove::<RigidBody>();
        commands.spawn((
            ChildOf(entity),
            crate::setup::cutscene::CutsceneWait,
            Transform::from_translation(Vec3::new(0.0, 0.0, -0.1)),
            canum_fx::splash::Splash {
                color: Color::linear_rgba(0.5, 1.0, 1.0, 0.8),
                duration: Duration::from_secs_f32(0.5),
                number: 100,
            },
        ));
        commands.spawn((
            ChildOf(entity),
            crate::setup::cutscene::CutsceneWait,
            canum_res::sound::Sound::new("Death"),
        ));
        commands.insert_resource(crate::setup::cutscene::CutsceneNext::new(
            crate::setup::StartSession {
                fight: "LobbySelect".to_owned(),
            },
        ));
    }
    next_state.set(crate::setup::GameState::Cutscene);
}

fn update_save(
    _event: On<PlayerFail>,
    mut save: ResMut<Save>,
    fight: Res<State<crate::setup::Fight>>,
) {
    let progress = save
        .progress
        .boss_progress
        .entry(fight.get().0.clone())
        .or_default();
    progress.fail_times += 1;
}
