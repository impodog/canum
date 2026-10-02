use super::*;

pub(super) struct DefeatPlugin;

impl Plugin for DefeatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(RULER_STATE.clone()), |mut commands: Commands| {
            canum_fx::session_observers!(
                commands,
                change_ruler_animation,
                ruler_fade_out_start,
                on_defeat_sequence
            );
        });
        app.add_systems(FixedUpdate, ruler_fade_out.in_set(RulerSet));
        app.add_systems(
            OnEnter(GET_SECONDARY_WEAPON_STATE.clone()),
            init_get_secondary_waepon,
        );
    }
}

#[derive(Event)]
pub struct RulerDefeated;

#[derive(Event, Default)]
struct RulerFadeOut;
#[derive(Event, Default)]
struct RulerDefeatSequence;

fn change_ruler_animation(
    _event: On<RulerDefeated>,
    mut animation: Single<&mut Animation, With<RulerBoss>>,
    background: Single<Entity, With<entry::UnderBackground>>,
    q_sound: Query<Entity, With<Sound>>,
    mut commands: Commands,
) {
    animation.size = vec2(128.0, 32.0);
    animation.size *= 0.9;
    animation.replace("Ruler_Breaks", false, None);
    for entity in q_sound.iter() {
        commands.entity(entity).try_despawn();
    }
    commands.spawn(Sound::new("Gen_GrandDefeat"));
    commands.entity(background.entity()).despawn();
    commands.spawn((SessionOnly, controls::OverrideMainControls));
    canum_fx::wait_then_trigger!(use commands, RulerFadeOut, 2.5);
    canum_fx::wait_then_trigger!(use commands, RulerDefeatSequence, 4.0);
}

#[derive(Component, Deref, DerefMut)]
struct FadeOut(Timer);
impl Default for FadeOut {
    fn default() -> Self {
        FadeOut(Timer::from_seconds(1.0, TimerMode::Once))
    }
}
fn ruler_fade_out_start(
    _event: On<RulerFadeOut>,
    mut commands: Commands,
    ruler: Single<Entity, With<RulerBoss>>,
) {
    commands.entity(ruler.entity()).insert(FadeOut::default());
}
fn ruler_fade_out(
    mut commands: Commands,
    mut fade_out: Single<(Entity, &mut Sprite, &mut FadeOut)>,
    time: Res<Time>,
) {
    if fade_out.2.tick(time.delta()).just_finished() {
        commands.entity(fade_out.0).try_despawn();
    } else {
        let alpha = fade_out.2.fraction_remaining();
        fade_out.1.color.set_alpha(alpha);
    }
}

fn on_defeat_sequence(_event: On<RulerDefeatSequence>, mut commands: Commands) {
    commands.spawn(Sound::new("Gen_MajorVictory"));
    commands.spawn(setup::cutscene::PureColorCutscene {
        transition: canum_fx::transition::PureColor {
            destroy: None,
            duration: Duration::from_secs_f32(8.0),
            color: Color::BLACK,
            remove_self: true,
        },
        fight: "_GetSecondaryWeapon".to_owned(),
    });
}

fn init_get_secondary_waepon(
    mut save: ResMut<Save>,
    mut commands: Commands,
    lang: Res<Lang>,
    fonts: Res<canum_ui::Fonts>,
    controller_suffix: Res<controls::ControllerSuffix>,
) {
    if save.progress.weapon_slots >= 2 {
        commands.trigger(player::victory::UpdateTasks {
            fight: "Ruler".to_owned(),
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
    save.progress.weapon_slots = 2;
    save.progress.gained_weapons.insert("D_Laser".to_owned());
    save.progress.charms.has_offensive = true;
    commands.spawn((
        SessionOnly,
        Text2d::new(lang.get(&format!("Ui_GetSecondaryWeapon_{}", *controller_suffix))),
        TextColor::WHITE,
        Transform::from_translation(vec3(0.0, 50.0, 0.0)),
        TextFont {
            font: fonts.game.clone().into(),
            font_size: FontSize::Px(35.0),
            font_smoothing: bevy::text::FontSmoothing::None,
            ..default()
        },
    ));
    commands.spawn((SessionOnly, Observer::new(listen_get_dash_input)));
}

fn listen_get_dash_input(_event: On<setup::lobby::LobbyQuit>, mut commands: Commands) {
    commands.trigger(player::victory::UpdateTasks {
        fight: "Ruler".to_owned(),
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
