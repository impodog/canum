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
            .init_resource::<FightTime>()
            .init_resource::<PostStartSessionSynchronizer>()
            .init_resource::<InitializationComplete>();
        app.add_message::<StartSession>();
        app.register_required_components::<canum_res::background::Background, crate::SessionOnly>();
        app.register_required_components::<canum_res::sound::Music, crate::SessionOnly>();

        app.add_observer(setup_session_send_message);
        app.add_systems(FixedLast, setup_session);
        app.add_systems(FixedFirst, send_delayed_start_session_events);

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

/// Sent after `StartSession` (which spawns absolutely necessary entities), used for initializing values and adding counters.
#[derive(Event, Debug, Clone)]
pub struct StartSessionFirst {
    pub fight: String,
    pub player_entity: Entity,
}

/// Sent after responding to `StartSessionFirst`, used for making modifications.
#[derive(Event, Debug, Clone)]
pub struct StartSessionMiddle {
    pub fight: String,
    pub player_entity: Entity,
}

/// Sent after responding to `StartSessionFirst`, used for spawning addons.
#[derive(Event, Debug, Clone)]
pub struct StartSessionAction {
    pub fight: String,
    pub player_entity: Entity,
}

/// Sent after responding to `StartSessionMiddle` with extra information, used for spawning UIs.
#[derive(Event, Debug, Clone)]
pub struct StartSessionLast {
    pub fight: String,
    pub player_entity: Entity,
}

#[derive(Resource, Debug, Default)]
pub struct CurrentSession {
    // TODO
}

/// When the session changes, all entitied marked with this are deleted.
#[derive(Component, Default)]
pub struct SessionOnly;

/// When the session changes, remove all of its children.
#[derive(Component, Default)]
pub struct ChildSessionOnly;

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
    q_child_session_only: Query<Entity, With<ChildSessionOnly>>,
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

    info!("Entering fight: {}", event.fight);
    commands.insert_resource(InitializationComplete::default());

    let Ok(mut camera_transform) = q_camera.single_mut() else {
        return;
    };
    *camera_transform = Transform::default();

    // Despawn previous entities
    for entity in q_session_only.iter() {
        commands.entity(entity).despawn();
    }
    for entity in q_child_session_only.iter() {
        commands.entity(entity).despawn_children();
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
    match save.progress.selected_health.as_str() {
        "BasicHp" => {
            commands
                .entity(player)
                .insert(crate::player::health::IntegerHealth::default());
        }
        health => {
            warn!("Unknown player health {health}, using default");
            commands
                .entity(player)
                .insert(crate::player::health::IntegerHealth::default());
        }
    };
    commands.insert_resource(crate::player::PrimaryPlayer(player));
    commands.insert_resource(crate::player::RandomPlayer(player));
    commands.insert_resource(crate::projectile::ProjectileBounds(
        Rect::from_center_half_size(Vec2::ZERO, CONFIG.display.screen_size),
    ));

    commands.insert_resource(FightTime::default());

    game_state.set(GameState::Play);
    play_state.set(PlayState::Fighting);
    fight.set(Fight(event.fight.clone()));

    commands.insert_resource(PostStartSessionSynchronizer::default());

    commands.trigger(StartSessionFirst {
        fight: event.fight.clone(),
        player_entity: player,
    });
}

#[derive(Resource, Default)]
struct PostStartSessionSynchronizer {
    first_tick: bool,
    middle_sent: bool,
    action_sent: bool,
    last_sent: bool,
}
/// The universal marker for all setup completed in any fight. This takes about 5 frames.
/// Use it with the system generator: `in_stable_state`.
#[derive(Resource, Default, Deref)]
pub struct InitializationComplete(bool);

pub fn in_stable_state<S: States>(
    target_state: S,
) -> impl Fn(Res<InitializationComplete>, Res<State<S>>) -> bool + Clone + Sync {
    move |init_complete: Res<InitializationComplete>, state: Res<State<S>>| -> bool {
        init_complete.0 && *state.get() == target_state
    }
}

fn send_delayed_start_session_events(
    mut commands: Commands,
    mut synchronizer: ResMut<PostStartSessionSynchronizer>,
    mut init_completed: ResMut<InitializationComplete>,
    primary_player: Option<Res<crate::player::PrimaryPlayer>>,
    fight: Res<State<Fight>>,
) {
    if init_completed.0 {
        return;
    }
    let Some(primary_player) = primary_player else {
        return;
    };
    if !synchronizer.last_sent {
        if !synchronizer.action_sent {
            if !synchronizer.middle_sent {
                if synchronizer.first_tick {
                    commands.trigger(StartSessionMiddle {
                        fight: fight.get().0.clone(),
                        player_entity: primary_player.0,
                    });
                    synchronizer.middle_sent = true;
                } else {
                    synchronizer.first_tick = true;
                }
            } else {
                commands.trigger(StartSessionAction {
                    fight: fight.get().0.clone(),
                    player_entity: primary_player.0,
                });
                synchronizer.action_sent = true;
            }
        } else {
            commands.trigger(StartSessionLast {
                fight: fight.get().0.clone(),
                player_entity: primary_player.0,
            });
            synchronizer.last_sent = true;
        }
    } else {
        init_completed.0 = true;
    }
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
