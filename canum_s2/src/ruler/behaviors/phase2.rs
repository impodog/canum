use super::*;

pub(super) struct Phase2Plugin;

impl Plugin for Phase2Plugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<RulerPhase2>()
            .on_add(ruler_phase2_hook);
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct RulerPhase2;

fn ruler_phase2_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world.commands().entity(entity);
}
