mod enemies;
mod entry;
mod flashlight;

pub static PROJECTED_STATE: LazyLock<setup::Fight> =
    LazyLock::new(|| setup::Fight("Projected".to_owned()));

use crate::prelude::*;

pub(super) struct ProjectedPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, SystemSet)]
pub struct ProjectedSet;

impl Plugin for ProjectedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            flashlight::FlashlightPlugin,
            entry::EntryPlugin,
            enemies::EnemiesPlugin,
        ));
        app.configure_sets(
            FixedUpdate,
            ProjectedSet.run_if(in_state(PROJECTED_STATE.to_owned())),
        );
        app.add_systems(
            OnEnter(PROJECTED_STATE.clone()),
            |mut commands: Commands| {
                commands.spawn((SessionOnly, Observer::new(projected_all_defeated)));
                commands.spawn((SessionOnly, Observer::new(projected_victory)));
            },
        );
    }
}

#[derive(Component, Default)]
#[require(SessionOnly, health::Friendly(false), player::victory::DefeatToWin)]
pub struct ProjectedMainEntity;

/// Sent immediately when all enemies has been defeated. This will trigger ProjectedVictory after a delay to actually finish the game.
#[derive(Event, Default)]
struct ProjectedAllDefeated;

#[derive(Event, Default)]
struct ProjectedVictory;
canum_fx::wait_then_trigger!(ProjectedVictoryTrigger, ProjectedVictory, 2.0);

fn projected_all_defeated(
    _event: On<ProjectedAllDefeated>,
    mut commands: Commands,
    music: Single<Entity, With<Music>>,
    mut main_entity: Single<&mut player::victory::DefeatToWin, With<ProjectedMainEntity>>,
) {
    main_entity.defeated = true;
    commands
        .entity(music.entity())
        .insert(canum_res::sound::FadeOut);
    commands
        .spawn(ProjectedVictoryTrigger)
        .observe(ProjectedVictoryTrigger::observer);
}

fn projected_victory(
    _event: On<ProjectedVictory>,
    main_entity: Single<Entity, With<ProjectedMainEntity>>,
    mut commands: Commands,
) {
    commands.entity(main_entity.entity()).despawn();
}
