use super::*;

pub(super) struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LobbySize>()
            .init_resource::<LobbyName>();
        app.add_observer(select_lobby).add_observer(enter_lobby);
        app.add_systems(
            FixedLast,
            camera_follow_player.run_if(in_state(PlayState::Lobby)),
        );
        app.add_systems(
            FixedLast,
            (dump_save_when_lobby_changed, change_window_title),
        );
    }
}

#[derive(Resource, Debug, Clone, Default, Deref, DerefMut)]
pub struct LobbySize(pub Vec2);

#[derive(Resource, Debug, Clone, Default, Deref, DerefMut)]
pub struct LobbyName(pub String);

/// Returns the position of the player when pressed 'confirm'.
#[derive(Event, Debug)]
pub struct LobbySelect {
    pub position: Vec2,
}

/// Returns the position of player when pressed 'shop'.
#[derive(Event, Debug)]
pub struct LobbyShop {
    pub position: Vec2,
}

/// Used by detailing lobby implementations.
#[derive(Event, Default)]
pub struct LobbyQuit;

fn select_lobby(event: On<StartSessionFirst>, mut commands: Commands, save: Res<Save>) {
    if event.fight != "LobbySelect" {
        return;
    }
    commands.trigger(StartSession {
        fight: save.progress.current_lobby.clone(),
    });
}

#[allow(clippy::single_match)]
fn enter_lobby(
    event: On<StartSessionFirst>,
    mut commands: Commands,
    save: Res<Save>,
    mut q_player: Query<&mut Transform, With<crate::player::Player>>,
    mut q_boundary: Query<Entity, With<Boundaries>>,
    mut play_state: ResMut<NextState<super::PlayState>>,
) {
    fn lobby_displacement(lobby_size: Vec2) -> Transform {
        Transform::from_translation(Vec3::new(lobby_size.x * 0.5, lobby_size.y * 0.5, -24.37))
    }

    let mut lobby_size = match event.fight.as_str() {
        "Gate" => {
            let Some(stage_details) = CONFIG.values.stage.get(event.fight.as_str()) else {
                return;
            };
            commands.spawn((
                SessionOnly,
                Animation::new(format!("{}_Lobby", event.fight), stage_details.full_size)
                    .with_color(Color::WHITE.with_alpha(0.8)),
                lobby_displacement(stage_details.full_size),
            ));
            stage_details.full_size
        }
        _ => {
            return;
        }
    };
    commands.insert_resource(LobbyName(event.fight.clone()));

    let extend_boundary: Vec2 = Vec2::new(
        CONFIG.display.half_virtual_size.0,
        CONFIG.display.half_virtual_size.1,
    );

    // Despawn previous and spawn new boundaries.
    for entity in q_boundary.iter_mut() {
        commands.entity(entity).despawn();
    }
    commands.spawn((
        Boundaries,
        Collider::rectangle(32.0, lobby_size.y),
        Transform::from_translation(Vec3::new(
            lobby_size.x + Boundaries::BOUNDARY_THICKNESS_HALF,
            lobby_size.y * 0.5,
            0.0,
        )),
    ));
    commands.spawn((
        Boundaries,
        Collider::rectangle(32.0, lobby_size.y),
        Transform::from_translation(Vec3::new(
            -Boundaries::BOUNDARY_THICKNESS_HALF,
            lobby_size.y * 0.5,
            0.0,
        )),
    ));
    commands.spawn((
        Boundaries,
        Collider::rectangle(lobby_size.x, 32.0),
        Transform::from_translation(Vec3::new(
            lobby_size.x * 0.5,
            lobby_size.y + Boundaries::BOUNDARY_THICKNESS_HALF,
            0.0,
        )),
    ));
    commands.spawn((
        Boundaries,
        Collider::rectangle(lobby_size.x, 32.0),
        Transform::from_translation(Vec3::new(
            lobby_size.x * 0.5,
            -Boundaries::BOUNDARY_THICKNESS_HALF,
            0.0,
        )),
    ));

    if let Some(stage_details) = CONFIG.values.stage.get(event.fight.as_str()) {
        for lock in stage_details.locks.iter() {
            let completed = lock.prereqs.iter().all(|prereq| {
                save.progress
                    .boss_progress
                    .get(prereq)
                    .is_some_and(|progress| progress.defeated)
            });
            if !completed {
                commands.spawn((
                    Boundaries,
                    Collider::rectangle(20.0, lobby_size.y),
                    Transform::from_translation(Vec3::new(lock.position, lobby_size.y * 0.5, 0.0)),
                ));
                lobby_size.x = lobby_size.x.min(lock.position);
            }
        }
    }

    commands.insert_resource(crate::projectile::ProjectileBounds(Rect {
        min: -extend_boundary,
        max: lobby_size + extend_boundary,
    }));
    commands.insert_resource(LobbySize(lobby_size));

    play_state.set(super::PlayState::Lobby);
    for mut transform in q_player.iter_mut() {
        transform.translation.x = save.progress.lobby_position.x;
        transform.translation.y = save.progress.lobby_position.y;
    }
}

fn camera_follow_player(
    mut q_camera: Query<&mut Transform, With<canum_res::camera::PixelCamera>>,
    q_player: Query<&GlobalTransform>,
    primary_player: Res<crate::player::PrimaryPlayer>,
    lobby_size: Res<LobbySize>,
) {
    fn safe_clamp(value: f32, min: f32, max: f32) -> f32 {
        if min > max || min.is_nan() || max.is_nan() {
            value
        } else {
            value.clamp(min, max)
        }
    }
    let Ok(player_transform) = q_player.get(primary_player.0) else {
        return;
    };
    let player_transform = player_transform.translation();
    let Ok(mut camera_transform) = q_camera.single_mut() else {
        return;
    };
    let new_x = safe_clamp(
        player_transform.x,
        CONFIG.display.half_virtual_size.0,
        lobby_size.x - CONFIG.display.half_virtual_size.0,
    );
    let new_y = safe_clamp(
        player_transform.y,
        CONFIG.display.half_virtual_size.1,
        lobby_size.y - CONFIG.display.half_virtual_size.1,
    );
    camera_transform.translation.x = new_x;
    camera_transform.translation.y = new_y;
}

#[allow(clippy::too_many_arguments)]
fn dump_save_when_lobby_changed(
    mut commands: Commands,
    q_player: Query<&GlobalTransform>,
    primary_player: Option<Res<crate::player::PrimaryPlayer>>,
    name: Res<LobbyName>,
    mut save: ResMut<Save>,
    play_state: Res<State<PlayState>>,
    game_state: Res<State<GameState>>,
    lobby_name: Res<LobbyName>,
) {
    if *play_state.get() != PlayState::Lobby {
        return;
    }
    let Some(primary_player) = primary_player else {
        return;
    };
    let exit_lobby = game_state.is_changed() && *game_state.get() == GameState::Cutscene;
    let enter_lobby = lobby_name.is_changed();
    if enter_lobby || exit_lobby {
        let Ok(player) = q_player.get(primary_player.0) else {
            return;
        };
        save.progress.current_lobby = name.0.clone();
        save.progress.lobby_position = player.translation().xy();
        commands.run_system(*canum_save::WRITE_SAVE.get().unwrap());
    }
}

fn change_window_title(
    mut title: ResMut<canum_res::window::WindowTitle>,
    lang: Res<Lang>,
    lobby_name: Res<LobbyName>,
) {
    if lobby_name.is_changed() {
        title.0 = lang
            .get(&format!("{}_WindowTitle", lobby_name.0))
            .to_owned();
    }
}

/// Clone marks of this boss from the save.
pub fn get_marks(save: &Save, name: &str) -> Vec<String> {
    save.progress
        .boss_progress
        .get(name)
        .map(|boss_progress| {
            boss_progress
                .tasks
                .iter()
                .map(|task| format!("Mark_{task}"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
