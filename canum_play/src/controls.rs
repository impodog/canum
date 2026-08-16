use crate::prelude::*;

pub(super) struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (keyboard_controls, connect_gamepads, gamepad_controls),
        );
        app.init_resource::<GamepadArrowEmulate>()
            .init_resource::<ControllerSuffix>();
    }
}

/// For some UI text related to controllers, this suffix is added to display different content accordingly.
/// e.g. "Keyboard" "Gamepad"
#[derive(Resource, Debug, Deref, DerefMut)]
pub struct ControllerSuffix(&'static str);
impl Default for ControllerSuffix {
    fn default() -> Self {
        Self("Keyboard")
    }
}
impl std::fmt::Display for ControllerSuffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Add this marker to UI panels to disable all main controls(player, lobby, shop).
#[derive(Default, Component)]
pub struct OverrideMainControls;

fn keyboard_controls(
    mut commands: Commands,
    primary_player: Option<Res<crate::player::PrimaryPlayer>>,
    save: Res<Save>,
    key: Res<ButtonInput<KeyCode>>,
    q_player: Query<(&GlobalTransform, &crate::player::attack::Weapons)>,
    override_main_controls: Query<(), With<OverrideMainControls>>,
) {
    let override_main_controls = override_main_controls.iter().next().is_some();

    let Some(primary_player) = primary_player.map(|player| **player) else {
        return;
    };
    let Ok((transform, weapons)) = q_player.get(primary_player) else {
        return;
    };
    let position = transform.translation().xy();

    let mut direction = Vec2::default();
    if key.pressed(save.keyboard.move_right) {
        direction.x += 1.0;
    }
    if key.pressed(save.keyboard.move_up) {
        direction.y += 1.0;
    }
    if key.pressed(save.keyboard.move_left) {
        direction.x -= 1.0;
    }
    if key.pressed(save.keyboard.move_down) {
        direction.y -= 1.0;
    }
    if !override_main_controls {
        if direction.length_squared() > 1e-8 {
            commands.trigger(crate::player::PlayerMove {
                entity: primary_player,
                rot: direction.to_angle(),
                mult: 1.0,
            });
            if save.progress.unlocked_dash && key.pressed(save.keyboard.dash) {
                commands.trigger(crate::movements::StartDash {
                    entity: primary_player,
                    base_velocity: direction.normalize(),
                })
            }
        }
        if let Some(primary_weapon) = weapons.first().copied().flatten() {
            if key.pressed(save.keyboard.primary_attack) {
                commands.trigger(crate::player::attack::Attack {
                    entity: primary_weapon,
                });
            } else if key.just_released(save.keyboard.primary_attack) {
                commands.trigger(crate::player::attack::AttackRelease {
                    entity: primary_weapon,
                });
            }
        }
        if weapons.len() >= 2
            && let Some(secondary_weapon) = weapons.last().copied().flatten()
        {
            if key.pressed(save.keyboard.secondary_attack) {
                commands.trigger(crate::player::attack::Attack {
                    entity: secondary_weapon,
                });
            } else if key.just_released(save.keyboard.secondary_attack) {
                commands.trigger(crate::player::attack::AttackRelease {
                    entity: secondary_weapon,
                });
            }
        }
        if key.just_pressed(save.keyboard.confirm) {
            commands.trigger(crate::setup::lobby::LobbySelect { position });
        }
    }

    if key.just_pressed(KeyCode::KeyS) {
        commands.trigger(crate::setup::lobby::LobbyShop { position });
    }
    if key.any_just_pressed([KeyCode::Escape, KeyCode::Backspace]) {
        commands.trigger(crate::setup::lobby::LobbyQuit);
    }
}

#[derive(Component, Default)]
pub struct MainGamepad;

fn connect_gamepads(
    q_gamepad: Query<Entity, With<MainGamepad>>,
    q_gamepads: Query<(Entity, &Gamepad)>,
    mut commands: Commands,
    mut suffix: ResMut<ControllerSuffix>,
) {
    if q_gamepad.iter().next().is_some() {
        return;
    }
    if let Some((entity, gamepad)) = q_gamepads.iter().next() {
        info!("Connected with gamepad {gamepad:?}");
        commands.entity(entity).insert(MainGamepad);
        suffix.0 = "Gamepad";
    } else if suffix.0 == "Gamepad" {
        suffix.0 = "Keyboard";
    }
}

#[derive(Resource, Default)]
pub struct GamepadArrowEmulate {
    pub trigger_time: Option<Duration>,
}

#[derive(Debug, Deref, DerefMut)]
struct GamepadTriggerInterval(Timer);
impl Default for GamepadTriggerInterval {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Once))
    }
}

#[allow(clippy::too_many_arguments)]
fn gamepad_controls(
    mut commands: Commands,
    primary_player: Option<Res<crate::player::PrimaryPlayer>>,
    q_player: Query<(&GlobalTransform, &crate::player::attack::Weapons)>,
    save: Res<Save>,

    mut gamepad: Single<&mut Gamepad, With<MainGamepad>>,
    mut arrow: ResMut<GamepadArrowEmulate>,

    override_main_controls: Query<(), With<OverrideMainControls>>,
    time: Res<Time>,
    mut interval: Local<GamepadTriggerInterval>,
) {
    const HOLD_DURATION: Duration = Duration::new(1, 0);

    let override_main_controls = override_main_controls.iter().next().is_some();
    let Some(primary_player) = primary_player.map(|player| **player) else {
        return;
    };
    let Ok((transform, weapons)) = q_player.get(primary_player) else {
        return;
    };
    let position = transform.translation().xy();

    let mut direction = Vec2::ZERO;
    direction.x = gamepad.get(GamepadAxis::LeftStickX).unwrap();
    direction.y = gamepad.get(GamepadAxis::LeftStickY).unwrap();
    let direction_length_sq = direction.length_squared();

    if direction_length_sq >= 0.25 {
        let can_trigger = if let Some(trigger_time) = arrow.trigger_time {
            if time.elapsed() > trigger_time + HOLD_DURATION {
                interval.tick(time.delta()).just_finished()
            } else {
                interval.reset();
                false
            }
        } else {
            arrow.trigger_time = Some(time.elapsed());
            true
        };
        if can_trigger {
            use std::f32::consts::*;
            let angle = canum_fx::math::normalize_angle(direction.to_angle());
            if (0.0..FRAC_PI_4).contains(&angle) || (TAU - FRAC_PI_4..TAU).contains(&angle) {
                gamepad.digital_mut().press(GamepadButton::DPadRight);
            } else if (FRAC_PI_4..FRAC_PI_4 * 3.0).contains(&angle) {
                gamepad.digital_mut().press(GamepadButton::DPadUp);
            } else if (FRAC_PI_4 * 3.0..FRAC_PI_4 * 5.0).contains(&angle) {
                gamepad.digital_mut().press(GamepadButton::DPadLeft);
            } else {
                gamepad.digital_mut().press(GamepadButton::DPadDown);
            }
        }
    }

    if !override_main_controls {
        if direction_length_sq >= 1e-2 {
            commands.trigger(crate::player::PlayerMove {
                entity: primary_player,
                rot: direction.to_angle(),
                mult: direction.length(),
            });
            if save.progress.unlocked_dash && gamepad.pressed(save.gamepad.dash) {
                commands.trigger(crate::movements::StartDash {
                    entity: primary_player,
                    base_velocity: direction.normalize(),
                })
            }
        }

        if let Some(primary_weapon) = weapons.first().copied().flatten() {
            if gamepad.pressed(save.gamepad.primary_attack) {
                commands.trigger(crate::player::attack::Attack {
                    entity: primary_weapon,
                });
            } else if gamepad.just_released(save.gamepad.primary_attack) {
                commands.trigger(crate::player::attack::AttackRelease {
                    entity: primary_weapon,
                });
            }
        }
        if weapons.len() >= 2
            && let Some(secondary_weapon) = weapons.last().copied().flatten()
        {
            if gamepad.pressed(save.gamepad.secondary_attack) {
                commands.trigger(crate::player::attack::Attack {
                    entity: secondary_weapon,
                });
            } else if gamepad.just_released(save.gamepad.secondary_attack) {
                commands.trigger(crate::player::attack::AttackRelease {
                    entity: secondary_weapon,
                });
            }
        }
        if gamepad.just_pressed(save.gamepad.confirm) {
            commands.trigger(crate::setup::lobby::LobbySelect { position });
        }
    }

    if gamepad.just_pressed(save.gamepad.shop) {
        commands.trigger(crate::setup::lobby::LobbyShop { position });
    }
    if gamepad.just_pressed(save.gamepad.cancel) {
        commands.trigger(crate::setup::lobby::LobbyQuit);
    }
}
