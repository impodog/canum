use crate::prelude::*;

pub(super) struct VictoryPlugin;

impl Plugin for VictoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(update_player_status)
            .add_observer(update_save);
        app.world_mut()
            .register_component_hooks::<DefeatToWin>()
            .on_remove(|mut world, HookContext { entity, .. }| {
                let defeat_to_win = world.get::<DefeatToWin>(entity).unwrap();
                if defeat_to_win.defeated {
                    world.commands().trigger(PlayerWin);
                }
            });
        app.add_systems(
            FixedLast,
            leave_victory.run_if(in_state(crate::setup::Fight("Victory".to_owned()))),
        );
    }
}

/// Marks the boss entity, after its defeat, the player automatically wins.
#[derive(Component, Default)]
pub struct DefeatToWin {
    pub defeated: bool,
}

/// Global event that the player defeated the boss.
#[derive(Event)]
pub struct PlayerWin;

fn update_player_status(
    _event: On<PlayerWin>,
    mut commands: Commands,
    q_player: Query<Entity, With<crate::player::Player>>,
    mut next_state: ResMut<NextState<crate::setup::GameState>>,
    state: Res<State<crate::setup::GameState>>,
) {
    if *state.get() == crate::setup::GameState::Cutscene {
        return;
    }
    for entity in q_player.iter() {
        commands
            .entity(entity)
            .remove::<Collider>()
            .remove::<RigidBody>();
        commands.spawn((
            ChildOf(entity),
            crate::setup::CutsceneWait,
            canum_res::sound::Sound::new("Victory"),
        ));
        commands.insert_resource(crate::setup::CutsceneNext {
            event: crate::setup::StartSession {
                fight: "Victory".to_owned(),
            },
        });
    }
    next_state.set(crate::setup::GameState::Cutscene);
}

fn update_save(
    _event: On<PlayerWin>,
    mut save: ResMut<Save>,
    mut window_title: ResMut<canum_res::window::WindowTitle>,
    fight: Res<State<crate::setup::Fight>>,
    any_hits: Res<super::tracking::AnyHits>,
    lang: Res<Lang>,
) {
    let progress = save
        .progress
        .boss_progress
        .entry(fight.get().0.clone())
        .or_default();
    if !progress.defeated {
        progress.defeated = true;
        progress.tasks.insert("Completed".to_owned());
    }
    if !**any_hits {
        progress.tasks.insert("NoHits".to_owned());
    }
    window_title.0 = lang
        .get(&format!("{}_Victory_WindowTitle", fight.get().0))
        .to_owned();
}

/// Leaves the dummy victory fight once schedule reachs FixedLast.
fn leave_victory(mut commands: Commands) {
    commands.trigger(crate::setup::StartSession {
        fight: "LobbySelect".to_owned(),
    });
}
