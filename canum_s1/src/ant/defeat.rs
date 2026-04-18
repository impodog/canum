use crate::prelude::*;

pub(super) struct DefeatPlugin;

impl Plugin for DefeatPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<AntDefeat>()
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
pub struct AntDefeat;

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
        setup::CutsceneWait,
        canum_fx::splash::Splash {
            color: Color::srgba(0.1, 0.3, 0.3, 0.5),
            duration: Duration::from_secs_f32(0.25),
            number: 30,
        },
    ));
    if let Ok(mut animation) = q_animation.get_mut(event.entity) {
        animation.replace(
            "Ant_Defeat",
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
