use crate::prelude::*;

pub(super) struct VictoryPlugin;

impl Plugin for VictoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(update_player_status);
        app.world_mut()
            .register_component_hooks::<DefeatToWin>()
            .on_remove(|mut world, HookContext { entity, .. }| {
                let defeat_to_win = world.get::<DefeatToWin>(entity).unwrap();
                if defeat_to_win.defeated {
                    world.commands().trigger(PlayerWin);
                }
            });
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
    mut next_state: ResMut<NextState<crate::setup::PlayState>>,
    state: Res<State<crate::setup::PlayState>>,
) {
    if *state.get() == crate::setup::PlayState::Cutscene {
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
    next_state.set(crate::setup::PlayState::Cutscene);
}
