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
/// Sent after responding to `StartSession` with extra information, used for spawning UIs.
#[derive(Event, Debug)]
pub struct PostStartSession {
    pub health_entity: Entity,
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
    save: Res<Save>,
    mut commands: Commands,
) {
    const BOUNDARY_THICKNESS: f32 = 32.0;
    const BOUNDARY_THICKNESS_HALF: f32 = BOUNDARY_THICKNESS * 0.5;

    // Despawn previous entities
    for entity in q_session_only.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn boundaries to restrict player and enemy
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

    // Spawn player and its weapons, health
    let mut weapons = Vec::new();
    for weapon in save
        .progress
        .selected_weapons
        .iter()
        .take(save.progress.weapon_slots)
    {
        match weapon.as_str() {
            "Filed" => {
                let entity = commands.spawn(crate::player::attack::Filed::default()).id();
                weapons.push(Some(entity));
            }
            "None" | "" => {
                weapons.push(None);
            }
            _ => {
                warn!("Unknown player weapon {weapon}");
                weapons.push(None);
            }
        }
    }
    let player = commands
        .spawn((
            crate::player::Player,
            Animation::new("Cyan", Vec2::new(20.0, 20.0)),
            crate::player::attack::Weapons(weapons.clone()),
        ))
        .add_children(
            weapons
                .iter()
                .filter_map(|entity| *entity)
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .id();
    let health_entity = match save.progress.selected_health.as_str() {
        "BasicHp" => commands
            .entity(player)
            .insert(crate::player::health::IntegerHealth::default())
            .id(),
        health => {
            warn!("Unknown player health {health}, using default");
            commands
                .entity(player)
                .insert(crate::player::health::IntegerHealth::default())
                .id()
        }
    };
    commands.insert_resource(crate::player::PrimaryPlayer(player));

    commands.trigger(PostStartSession { health_entity });
}
