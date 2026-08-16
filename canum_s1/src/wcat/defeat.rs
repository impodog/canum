use super::*;

pub(super) struct DefeatPlugin;

impl Plugin for DefeatPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<WcatDefeat>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(on_defeat);
            });
        app.add_systems(OnEnter(WCAT_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(on_transition)));
        });
        app.add_systems(OnEnter(GET_DASH_STATE.clone()), init_get_dash);
        app.add_systems(
            FixedPreUpdate,
            listen_get_dash_input.run_if(in_state(GET_DASH_STATE.clone())),
        );
    }
}

#[derive(Component, Default)]
pub struct WcatDefeat;

#[derive(Event, Default)]
pub struct DefeatInform;
canum_fx::wait_then_trigger!(DefeatInformTrigger, DefeatInform, 3.0);

fn on_defeat(
    event: On<enemy::health::EnemyDefeated>,
    mut commands: Commands,
    mut q_animation: Query<&mut Animation>,
    q_music: Query<Entity, With<Music>>,
) {
    commands
        .entity(event.entity)
        .despawn_children()
        .remove::<Collider>()
        .remove::<RigidBody>();

    for entity in q_music.iter() {
        commands.entity(entity).despawn();
    }

    commands.spawn(canum_res::sound::Sound::new("Wcat_Bump").with_volume_add(5.0));
    if let Ok(mut animation) = q_animation.get_mut(event.entity) {
        animation.replace("Wcat_Defeat", false, None);
    }
    commands
        .spawn(DefeatInformTrigger)
        .observe(DefeatInformTrigger::observer);
}

fn on_transition(_event: On<DefeatInform>, mut commands: Commands) {
    commands.spawn(Sound::new("Wcat_Defeat"));
    commands.spawn(setup::cutscene::PureColorCutscene {
        transition: canum_fx::transition::PureColor {
            destroy: None,
            duration: Duration::from_secs_f32(8.0),
            color: Color::BLACK,
            remove_self: true,
        },
        fight: "_GetDash".to_owned(),
    });
}

fn init_get_dash(
    mut save: ResMut<Save>,
    mut commands: Commands,
    lang: Res<Lang>,
    fonts: Res<canum_ui::Fonts>,
    controller_suffix: Res<controls::ControllerSuffix>,
) {
    if save.progress.unlocked_dash {
        // ! Must update tasks for wcat even if not obtaining dash.
        commands.trigger(player::victory::UpdateTasks {
            fight: "Wcat".to_owned(),
        });
        commands.spawn(setup::cutscene::PureColorCutscene {
            transition: canum_fx::transition::PureColor {
                destroy: None,
                duration: Duration::from_secs_f32(2.0),
                color: Color::BLACK,
                remove_self: true,
            },
            fight: "LobbySelect".to_owned(),
        });
        return;
    }
    save.progress.unlocked_dash = true;
    commands.spawn((
        SessionOnly,
        Text2d::new(lang.get(&format!("Ui_GetDash_{}", *controller_suffix))),
        TextColor::WHITE,
        Transform::from_translation(vec3(0.0, 100.0, 0.0)),
        TextFont {
            font: fonts.game.clone().into(),
            font_size: FontSize::Px(35.0),
            font_smoothing: bevy::text::FontSmoothing::None,
            ..default()
        },
    ));
}

fn listen_get_dash_input(keyboard: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keyboard.any_just_pressed([KeyCode::Escape, KeyCode::Backspace, KeyCode::Enter]) {
        commands.trigger(player::victory::UpdateTasks {
            fight: "Wcat".to_owned(),
        });
        commands.spawn(setup::cutscene::PureColorCutscene {
            transition: canum_fx::transition::PureColor {
                destroy: None,
                duration: Duration::from_secs_f32(2.0),
                color: Color::BLACK,
                remove_self: true,
            },
            fight: "LobbySelect".to_owned(),
        });
    }
}
