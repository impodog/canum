use crate::prelude::*;
use bevy::ecs::lifecycle::HookContext;

pub(super) struct VisualPlugin;

impl Plugin for VisualPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<Shadow>()
            .on_insert(|mut world, HookContext { entity, .. }| {
                let Some(shadow) = world.get::<Shadow>(entity) else {
                    return;
                };
                let length = shadow.0;
                let Some(mut animation) = world.get_mut::<canum_res::Animation>(entity) else {
                    return;
                };
                *animation = canum_res::Animation::new("Shadow", vec2(length, length * 0.5));
                let Some(mut transform) = world.get_mut::<Transform>(entity) else {
                    return;
                };
                transform.translation.y = -length * 0.5;
            });
    }
}

/// A shadow of given length.
#[derive(Component, Debug, Clone)]
#[require(canum_res::Animation, Transform::from_translation(vec3(0.0, 0.0, -0.1)))]
pub struct Shadow(pub f32);
impl Default for Shadow {
    fn default() -> Self {
        Self(32.0)
    }
}
