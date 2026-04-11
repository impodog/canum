use crate::prelude::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<AntEntry>()
            .on_add(|mut world, HookContext { entity, .. }| {
                let ant = world.get::<AntEntry>(entity).unwrap().0;
                world
                    .commands()
                    .entity(ant)
                    .insert(enemy::movements::Displacement {
                        displace: vec2(-400.0, 0.0),
                        duration: Duration::from_secs_f32(0.8),
                        notify: Some(entity),
                    });
                world
                    .commands()
                    .entity(entity)
                    .observe(ant_entry_despawn_self);
            });
        app.add_systems(
            OnEnter(super::ANT_STATE.clone()),
            |mut commands: Commands| {
                commands.spawn((SessionOnly, Observer::new(spawn_ant_title)));
            },
        );
    }
}

#[derive(Component)]
pub struct AntEntry(pub Entity);

#[derive(Event)]
pub struct AntEntryComplete;

fn ant_entry_despawn_self(
    event: On<enemy::movements::DisplacementComplete>,
    mut commands: Commands,
    q_ant_entry: Query<&AntEntry>,
    mut q_ant: Query<&mut Animation>,
) {
    let Ok(ant_entry) = q_ant_entry.get(event.entity) else {
        return;
    };
    commands
        .entity(ant_entry.0)
        .remove::<projectile::NoCollideBoundary>();
    let Ok(mut animation) = q_ant.get_mut(ant_entry.0) else {
        return;
    };
    animation.replace("Ant_Static", false, None);
    commands.entity(event.entity).despawn();
    commands.trigger(AntEntryComplete);
}

fn spawn_ant_title(
    _event: On<AntEntryComplete>,
    mut commands: Commands,
    q_bottom_left: Query<Entity, With<canum_ui::BottomLeft>>,
    fonts: Res<canum_ui::Fonts>,
    lang: Res<Lang>,
) {
    let Ok(bottom_left) = q_bottom_left.single() else {
        return;
    };
    commands.spawn((
        ChildOf(bottom_left),
        canum_ui::text::popup_title(
            fonts.title.clone(),
            lang.get("Ant_BossTitle"),
            Duration::from_secs(1),
        ),
    ));
}
