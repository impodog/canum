use crate::prelude::*;

pub(super) struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<ParentColliderDisabled>()
            .on_add(
                |mut world, bevy::ecs::lifecycle::HookContext { entity, .. }| {
                    let Some(parent) = world.get::<ChildOf>(entity).cloned() else {
                        return;
                    };
                    world
                        .commands()
                        .entity(parent.0)
                        .entry::<DisablingChildren>()
                        .and_modify(move |mut children| children.children.push(entity))
                        .or_insert(DisablingChildren {
                            children: vec![entity],
                        });
                },
            );
        app.add_systems(
            FixedPostUpdate,
            update_disabling_children.in_set(PhysicsSystems::First),
        );
    }
}

/// Disable the parent's collider as long as this component exists.
#[derive(Component, Default)]
pub struct ParentColliderDisabled;

/// Look for these children if they have removed the component.
/// This is automatically added when a disabling child first exists, and automatically removed if all such children are removed.
#[derive(Component, Default)]
struct DisablingChildren {
    children: Vec<Entity>,
}

fn update_disabling_children(
    mut q_parent: Query<(Entity, &mut DisablingChildren, Option<&ColliderDisabled>)>,
    q_child: Query<(), With<ParentColliderDisabled>>,
    commands: ParallelCommands,
) {
    q_parent
        .par_iter_mut()
        .for_each(|(entity, mut children, collider_disabled)| {
            let new_children = children
                .children
                .drain(..)
                .filter(|child| q_child.get(*child).is_ok())
                .collect::<Vec<_>>();
            if new_children.is_empty() {
                commands.command_scope(|mut commands| {
                    commands
                        .entity(entity)
                        .remove::<DisablingChildren>()
                        .try_remove::<ColliderDisabled>();
                });
            } else {
                children.children = new_children;
                if collider_disabled.is_none() {
                    commands.command_scope(|mut commands| {
                        commands.entity(entity).try_insert(ColliderDisabled);
                    });
                }
            }
        });
}
