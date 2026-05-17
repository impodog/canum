use bevy::time::Stopwatch;

use crate::prelude::*;

pub mod cutscene;
pub mod lobby;
pub mod shop;

pub(super) struct SetupPlugin;
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            lobby::LobbyPlugin,
            shop::ShopPlugin,
            cutscene::CutscenePlugin,
        ));
        app.init_resource::<CurrentSession>()
            .init_resource::<FightTime>();
        app.add_message::<StartSession>();
        app.register_required_components::<canum_res::background::Background, crate::SessionOnly>();
        app.register_required_components::<canum_res::sound::Music, crate::SessionOnly>();

        app.add_observer(setup_session_send_message);
        app.add_systems(FixedLast, setup_session);

        app.init_state::<GameState>()
            .init_state::<PlayState>()
            .init_state::<Fight>();
        app.add_systems(PreUpdate, tick_fight_time);
        app.add_observer(change_music_when_win)
            .add_observer(change_music_when_lose);
    }
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Play,
    Cutscene,
}

/// Substates of `GameState::Play`.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlayState {
    #[default]
    Fighting,
    Lobby,
    Shop,
}

#[derive(States, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Deref, DerefMut)]
pub struct Fight(pub String);

#[derive(Resource, Debug, Default, Deref, DerefMut)]
pub struct FightTime(pub Stopwatch);

/// External callers should trigger this. The message is only used within setup.
#[derive(Event, Message, Debug, Clone)]
pub struct StartSession {
    pub fight: String,
}
/// Sent after responding to `StartSession` with extra information, used for spawning UIs.
#[derive(Event, Debug, Clone)]
pub struct PostStartSession {
    pub fight: String,
    pub player_entity: Entity,
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
impl Boundaries {
    pub const BOUNDARY_THICKNESS: f32 = 32.0;
    pub const BOUNDARY_THICKNESS_HALF: f32 = Self::BOUNDARY_THICKNESS * 0.5;
}

/// This makes sure that all commands are run before setup_session, ensuring correct `SessionOnly` despawn.
fn setup_session_send_message(event: On<StartSession>, mut writer: MessageWriter<StartSession>) {
    writer.write(event.clone());
}

#[allow(clippy::too_many_arguments)]
fn setup_session(
    mut reader: MessageReader<StartSession>,
    q_session_only: Query<Entity, With<SessionOnly>>,
    save: Res<Save>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    mut play_state: ResMut<NextState<PlayState>>,
    mut fight: ResMut<NextState<Fight>>,
    mut q_camera: Query<&mut Transform, With<canum_res::camera::PixelCamera>>,
) {
    let Some(event) = reader.read().last() else {
        return;
    };

    let Ok(mut camera_transform) = q_camera.single_mut() else {
        return;
    };
    *camera_transform = Transform::default();

    // Despawn previous entities
    for entity in q_session_only.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn boundaries to restrict player and enemy
    commands.spawn((
        Boundaries,
        Collider::rectangle(32.0, CONFIG.display.virtual_size.1 as f32),
        Transform::from_translation(Vec3::new(
            CONFIG.display.half_virtual_size.0 + Boundaries::BOUNDARY_THICKNESS_HALF,
            0.0,
            0.0,
        )),
    ));
    commands.spawn((
        Boundaries,
        Collider::rectangle(32.0, CONFIG.display.virtual_size.1 as f32),
        Transform::from_translation(Vec3::new(
            -CONFIG.display.half_virtual_size.0 - Boundaries::BOUNDARY_THICKNESS_HALF,
            0.0,
            0.0,
        )),
    ));
    commands.spawn((
        Boundaries,
        Collider::rectangle(CONFIG.display.virtual_size.0 as f32, 32.0),
        Transform::from_translation(Vec3::new(
            0.0,
            CONFIG.display.half_virtual_size.1 + Boundaries::BOUNDARY_THICKNESS_HALF,
            0.0,
        )),
    ));
    commands.spawn((
        Boundaries,
        Collider::rectangle(CONFIG.display.virtual_size.0 as f32, 32.0),
        Transform::from_translation(Vec3::new(
            0.0,
            -CONFIG.display.half_virtual_size.1 - Boundaries::BOUNDARY_THICKNESS_HALF,
            0.0,
        )),
    ));

    // Spawn player and its weapons, health
    // Update: weapons are now handled in `canum_addons` crate.
    let mut weapons = Vec::new();
    for _weapon in save
        .progress
        .selected_weapons
        .iter()
        .take(save.progress.weapon_slots)
    {
        weapons.push(None);
    }
    let player = commands
        .spawn((
            crate::player::Player,
            Animation::new("Cyan", Vec2::new(18.0, 18.0)),
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
    commands.insert_resource(crate::player::RandomPlayer(player));
    commands.insert_resource(crate::projectile::ProjectileBounds(
        Rect::from_center_half_size(Vec2::ZERO, CONFIG.display.screen_size),
    ));

    commands.trigger(PostStartSession {
        fight: event.fight.clone(),
        player_entity: player,
        health_entity,
    });

    commands.insert_resource(FightTime::default());

    game_state.set(GameState::Play);
    play_state.set(PlayState::Fighting);
    fight.set(Fight(event.fight.clone()));
}

fn tick_fight_time(time: Res<Time>, mut fight_time: ResMut<FightTime>) {
    fight_time.tick(time.delta());
}

fn change_music_when_win(
    _event: On<crate::player::victory::PlayerWin>,
    q_music: Query<Entity, With<canum_res::sound::Music>>,
    commands: Commands,
) {
    change_music_when_win_or_lose(q_music, commands);
}
fn change_music_when_lose(
    _event: On<crate::player::failure::PlayerFail>,
    q_music: Query<Entity, With<canum_res::sound::Music>>,
    commands: Commands,
) {
    change_music_when_win_or_lose(q_music, commands);
}
fn change_music_when_win_or_lose(
    q_music: Query<Entity, With<canum_res::sound::Music>>,
    mut commands: Commands,
) {
    for entity in q_music.iter() {
        commands.entity(entity).insert(canum_res::sound::FadeOut);
    }
}
