use crate::prelude::*;

pub(super) struct CutscenePlugin;

impl Plugin for CutscenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedLast,
            wait_for_cutscene.run_if(in_state(crate::setup::GameState::Cutscene)),
        );
        app.add_systems(OnEnter(crate::setup::GameState::Cutscene), cutscene_delete);
        app.add_systems(FixedUpdate, init_pure_color_cutscene);
    }
}

#[derive(Resource, Debug, Clone)]
pub struct CutsceneNext {
    pub event: crate::setup::StartSession,
    /// Only when dirty, the cutscene transition event is sent, and the dirty flag will be set to false.
    dirty: bool,
}
impl CutsceneNext {
    pub fn new(event: crate::setup::StartSession) -> Self {
        Self { event, dirty: true }
    }
}

/// Instructs the cutscene state to wait for all these entities to despawn, then the next session may load.
#[derive(Component, Default)]
pub struct CutsceneWait;

fn wait_for_cutscene(
    mut commands: Commands,
    cutscene_next: Option<ResMut<CutsceneNext>>,
    q_wait: Query<(), With<CutsceneWait>>,
) {
    if let Some(mut cutscene_next) = cutscene_next
        && q_wait.iter().next().is_none()
        && cutscene_next.dirty
    {
        commands.trigger(cutscene_next.event.clone());
        cutscene_next.dirty = false;
    }
}

/// When the entering cutscene, all entitied marked with this are deleted.
/// This also adds `SessionOnly` as a weaker constraint.
#[derive(Component, Default)]
#[require(SessionOnly)]
pub struct CutsceneDelete;

fn cutscene_delete(mut commands: Commands, q_cutscene_delete: Query<Entity, With<CutsceneDelete>>) {
    for entity in q_cutscene_delete.iter() {
        if let Ok(mut commands) = commands.get_entity(entity) {
            commands.despawn();
        }
    }
}

/// Spawn this to enter a cutscene with that pure color effect and enter another session.
#[derive(Component, Debug, Clone)]
pub struct PureColorCutscene {
    pub transition: canum_fx::transition::PureColor,
    pub fight: String,
}

fn init_pure_color_cutscene(
    mut commands: Commands,
    q_pure_color: Query<(Entity, &PureColorCutscene), Without<canum_fx::transition::PureColor>>,
    mut game_state: ResMut<NextState<crate::setup::GameState>>,
    state: Res<State<crate::setup::GameState>>,
) {
    let mut has_cutscene = *state.get() == setup::GameState::Cutscene;
    for (entity, pure_color) in q_pure_color.iter() {
        if has_cutscene {
            commands.entity(entity).despawn();
            continue;
        }
        commands.spawn((ChildOf(entity), CutsceneWait));
        commands
            .entity(entity)
            .insert(pure_color.transition.clone())
            .observe(unleash_cutscene_pure_color_transition);
        commands.insert_resource(CutsceneNext::new(setup::StartSession {
            fight: pure_color.fight.clone(),
        }));
        game_state.set(setup::GameState::Cutscene);
        has_cutscene = true;
    }
}

fn unleash_cutscene_pure_color_transition(
    event: On<canum_fx::transition::PureColorHalfPoint>,
    mut commands: Commands,
    q_children: Query<&Children>,
    q_wait: Query<(), With<CutsceneWait>>,
) {
    let Ok(children) = q_children.get(event.entity) else {
        return;
    };
    for child in children.iter() {
        if q_wait.get(child).is_ok() {
            commands.entity(child).despawn();
        }
    }
}
