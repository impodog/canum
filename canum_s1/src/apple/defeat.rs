use crate::prelude::*;

pub(super) struct DefeatPlugin;

impl Plugin for DefeatPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<AppleDefeat>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(on_defeat)
                    .observe(on_animation_played);
            });
    }
}

#[derive(Component, Default)]
pub struct AppleDefeat;

fn on_defeat(
    event: On<enemy::health::EnemyDefeated>,
    mut commands: Commands,
    mut q_animation: Query<&mut Animation>,
) {
    commands
        .entity(event.entity)
        .despawn_children()
        .remove::<Collider>()
        .remove::<RigidBody>();
    commands.spawn((
        ChildOf(event.entity),
        setup::cutscene::CutsceneWait,
        canum_fx::splash::Splash {
            color: Color::linear_rgba(0.8, 0.1, 0.1, 0.5),
            duration: Duration::from_secs_f32(0.25),
            number: 40,
        },
    ));
    commands.spawn((
        ChildOf(event.entity),
        canum_res::sound::Sound::new("Apple_Defeat"),
    ));
    if let Ok(mut animation) = q_animation.get_mut(event.entity) {
        animation.replace(
            "Apple_Defeat",
            true,
            Some(AnimationInform {
                entity: event.entity,
                index: vec![0],
            }),
        );
    }
}
fn on_animation_played(event: On<AnimationComplete>, mut commands: Commands) {
    commands.entity(event.entity).despawn();
}
