use crate::prelude::*;

pub(super) struct DefeatPlugin;

impl Plugin for DefeatPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<TurfDefeat>()
            .on_add(|mut world, HookContext { entity, .. }| {
                let mut commands = world.commands();
                commands.entity(entity).observe(on_defeat);
            });
    }
}

#[derive(Component, Default)]
pub struct TurfDefeat;

fn on_defeat(
    _event: On<enemy::health::EnemyDefeated>,
    mut q_background: Query<&mut Animation, With<canum_res::background::Background>>,
) {
    let Ok(mut background) = q_background.single_mut() else {
        return;
    };
    background.replace("Turf_BackgroundDefeated", false, None);
}
