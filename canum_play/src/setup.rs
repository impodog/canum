use crate::prelude::*;

pub(super) struct SetupPlugin;
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentSession>();
        app.add_observer(setup_session);
    }
}

#[derive(Event, Debug)]
pub struct StartSession {
    // TODO
}

#[derive(Resource, Debug, Default)]
pub struct CurrentSession {
    // TODO
}

/// When the session changes, all entitied marked with this are deleted.
#[derive(Component, Default)]
pub struct SessionOnly;

/// Limits the moving range of entites. This is a invisible box.
#[derive(Component, Default)]
#[require(Collider, RigidBody::Static, Transform, SessionOnly)]
pub struct Boundaries;

fn setup_session(
    _event: On<StartSession>,
    q_session_only: Query<Entity, With<SessionOnly>>,
    mut commands: Commands,
) {
    const BOUNDARY_THICKNESS: f32 = 32.0;
    const BOUNDARY_THICKNESS_HALF: f32 = BOUNDARY_THICKNESS * 0.5;
    for entity in q_session_only.iter() {
        commands.entity(entity).despawn();
    }
    info!("SETUP");
    commands.spawn((
        Boundaries,
        Collider::rectangle(32.0, CONFIG.display.virtual_size.1 as f32),
        Transform::from_translation(Vec3::new(
            CONFIG.display.half_virtual_size.0 + BOUNDARY_THICKNESS_HALF,
            0.0,
            0.0,
        )),
    ));
    commands.spawn((
        Boundaries,
        Collider::rectangle(32.0, CONFIG.display.virtual_size.1 as f32),
        Transform::from_translation(Vec3::new(
            -CONFIG.display.half_virtual_size.0 - BOUNDARY_THICKNESS_HALF,
            0.0,
            0.0,
        )),
    ));
    commands.spawn((
        Boundaries,
        Collider::rectangle(CONFIG.display.virtual_size.0 as f32, 32.0),
        Transform::from_translation(Vec3::new(
            0.0,
            CONFIG.display.half_virtual_size.1 + BOUNDARY_THICKNESS_HALF,
            0.0,
        )),
    ));
    commands.spawn((
        Boundaries,
        Collider::rectangle(CONFIG.display.virtual_size.0 as f32, 32.0),
        Transform::from_translation(Vec3::new(
            0.0,
            -CONFIG.display.half_virtual_size.1 - BOUNDARY_THICKNESS_HALF,
            0.0,
        )),
    ));
}
