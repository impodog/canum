use std::collections::BTreeSet;

use crate::prelude::*;

pub(super) struct VictoryPlugin;

impl Plugin for VictoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(update_player_status)
            .add_observer(update_tasks);
        app.world_mut()
            .register_component_hooks::<DefeatToWin>()
            .on_remove(|mut world, HookContext { entity, .. }| {
                let defeat_to_win = world.get::<DefeatToWin>(entity).unwrap();
                let state = world.resource::<State<crate::setup::GameState>>();
                if defeat_to_win.defeated && *state.get() == crate::setup::GameState::Play {
                    world.commands().trigger(PlayerWin::default());
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
#[derive(Event, Default)]
pub struct PlayerWin {
    // For manual cutscenes instead of the default one.
    pub skip_cutscene: bool,
}

fn update_player_status(
    event: On<PlayerWin>,
    mut commands: Commands,
    q_player: Query<Entity, With<crate::player::Player>>,
    mut next_state: ResMut<NextState<crate::setup::GameState>>,
    state: Res<State<crate::setup::GameState>>,
    fight: Res<State<setup::Fight>>,
) {
    if *state.get() == crate::setup::GameState::Cutscene {
        return;
    }
    commands.trigger(UpdateTasks {
        fight: fight.get().0.clone(),
    });
    for entity in q_player.iter() {
        commands.entity(entity).remove::<Collider>();
        if !event.skip_cutscene {
            commands.spawn((
                ChildOf(entity),
                crate::setup::cutscene::CutsceneWait,
                canum_res::sound::Sound::new("Victory"),
            ));
        }
        commands.insert_resource(crate::setup::cutscene::CutsceneNext::new(
            crate::setup::StartSession {
                fight: "Victory".to_owned(),
            },
        ));
    }
    next_state.set(crate::setup::GameState::Cutscene);
}

#[derive(Event, Default, Debug, Clone, Deref, DerefMut)]
pub struct CompletedTasks {
    #[deref]
    pub tasks: BTreeSet<String>,
    // This is used for in-between states to update the correct tasks of the original fight.
    pub fight: String,
}

/// Calls for related code to update task of this fight name. Automatically called for normal victory states.
#[derive(Event, Debug, Clone)]
pub struct UpdateTasks {
    pub fight: String,
}

fn update_tasks(
    event: On<UpdateTasks>,
    mut commands: Commands,
    mut save: ResMut<Save>,
    mut window_title: ResMut<canum_res::window::WindowTitle>,
    any_hits: Res<super::tracking::AnyHits>,
    lang: Res<Lang>,
) {
    let progress = save
        .progress
        .boss_progress
        .entry(event.fight.clone())
        .or_default();
    let previous_tasks = progress.tasks.clone();

    if !progress.defeated {
        progress.defeated = true;
        progress.tasks.insert("Completed".to_owned());
    }
    if !**any_hits {
        progress.tasks.insert("NoHits".to_owned());
    }

    let completed_tasks = progress
        .tasks
        .difference(&previous_tasks)
        .cloned()
        .collect::<BTreeSet<_>>();
    commands.trigger(CompletedTasks {
        tasks: completed_tasks,
        fight: event.fight.clone(),
    });
    window_title.0 = lang
        .get_or_empty(&format!("{}_Victory_WindowTitle", event.fight))
        .to_owned();
}

/// Leaves the dummy victory fight once schedule reachs FixedLast.
fn leave_victory(mut commands: Commands) {
    commands.trigger(crate::setup::StartSession {
        fight: "LobbySelect".to_owned(),
    });
}
