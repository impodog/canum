use super::*;

pub(super) struct FlashlightPlugin;

impl Plugin for FlashlightPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<FlashlightOverlay>()
            .on_add(flashlight_hook);
    }
}

#[derive(Component, Clone, Copy)]
#[require(Transform, Visibility)]
pub struct FlashlightOverlay {
    pub amplify_ratio: f32,
}
impl Default for FlashlightOverlay {
    fn default() -> Self {
        Self { amplify_ratio: 1.0 }
    }
}

fn flashlight_hook(
    mut world: bevy::ecs::world::DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    let Some(FlashlightOverlay { amplify_ratio }) = world.get::<FlashlightOverlay>(entity).copied()
    else {
        return;
    };
    let size = vec2(400.0, 225.0) * amplify_ratio;
    let pure_sprite = Sprite {
        color: Color::BLACK,
        custom_size: Some(size),
        ..default()
    };
    world.commands().spawn((
        ChildOf(entity),
        Animation::new("Projected_Flashlight", size),
        Transform::from_translation(vec3(0.0, 0.0, 0.0)),
    ));
    for i in -3..=3 {
        for j in -3..=3 {
            if i != 0 || j != 0 {
                let position = vec2(i as f32 * size.x, j as f32 * size.y);
                world.commands().spawn((
                    ChildOf(entity),
                    pure_sprite.clone(),
                    Transform::from_translation(vec3(position.x, position.y, 0.0)),
                ));
            }
        }
    }
}
