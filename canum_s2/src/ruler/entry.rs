use super::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(RULER_STATE.clone()), setup_ruler);
        app.add_systems(
            FixedUpdate,
            (ruler_handle_title_movements, ruler_handle_title_fade_out).in_set(RulerSet),
        );
    }
}

/// After a short interval the dialogue will appear.
#[derive(Event, Default)]
pub struct RulerSpawnDialogue;
canum_fx::wait_then_trigger!(RulerSpawnDialogueTrigger, RulerSpawnDialogue, 0.5);

/// After the ruler dialogue, either of the titles will spawn. Only the first time it will be a dramatic big title.
#[derive(Event, Clone, Copy)]
pub enum RulerSpawnTitle {
    Normal,
    Big,
}

/// After the ruler big title, where player gain control.
#[derive(Event)]
pub struct RulerStart;

fn setup_ruler(
    mut commands: Commands,
    mut title: ResMut<canum_res::window::WindowTitle>,
    lang: Res<Lang>,
) {
    commands
        .spawn(RulerSpawnDialogueTrigger)
        .observe(RulerSpawnDialogueTrigger::observer);
    canum_fx::session_observers!(
        commands,
        ruler_spawn_dialogue,
        ruler_dialogue_complete,
        ruler_spawn_title,
        ruler_start,
    );
    title.0 = lang.get("Ruler_WindowTitle").to_owned();
}

fn ruler_spawn_dialogue(
    _event: On<RulerSpawnDialogue>,
    mut commands: Commands,
    bottom_center: Single<Entity, With<canum_ui::BottomCenter>>,
    mut save: ResMut<Save>,
    lang: Res<Lang>,
) {
    if save.progress.first_time("Ruler_Prefight") {
        commands.spawn((
            ChildOf(bottom_center.entity()),
            canum_ui::dialogue::Dialogue::from_config("Ruler_Prefight", lang),
        ));
    } else {
        commands.trigger(RulerSpawnTitle::Normal);
    }
}

fn ruler_dialogue_complete(
    _event: On<canum_ui::dialogue::DialogueComplete>,
    mut commands: Commands,
) {
    commands.trigger(RulerSpawnTitle::Big);
}

/// Marks the big title part so that the fade function can set their alpha.
#[derive(Component, Default)]
#[require(controls::OverrideMainControls)]
struct EntryBigTitle {
    move_vec: Vec2,
}

#[derive(Component, Default)]
struct EntryBigTitleMiddle;

#[derive(Component, Deref, DerefMut)]
struct TitleInTimer(Timer);
impl Default for TitleInTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(TITLE_IN_TIME, TimerMode::Once))
    }
}

#[derive(Component, Deref, DerefMut)]
struct TitleOutTimer(Timer);
impl Default for TitleOutTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(TITLE_OUT_TIME, TimerMode::Once))
    }
}

const TITLE_SIZE: Vec2 = vec2(400.0, 100.0);
const TITLE_ANGLE: f32 = 15.0_f32.to_radians();
const TITLE_Z: f32 = 25.0;
const TITLE_IN_TIME: f32 = 1.2;
const TITLE_OUT_TIME: f32 = 1.6;
const TITLE_COLOR: Color = Color::srgb(0.0, 1.0, 1.0);

fn ruler_spawn_title(
    event: On<RulerSpawnTitle>,
    mut commands: Commands,
    bottom_left: Single<Entity, With<canum_ui::BottomLeft>>,
    lang: Res<Lang>,
    fonts: Res<canum_ui::Fonts>,
) {
    match *event {
        RulerSpawnTitle::Normal => {
            commands.spawn((
                ChildOf(bottom_left.entity()),
                canum_ui::text::popup_title(
                    fonts.title.clone(),
                    lang.get("Ruler_BossTitle"),
                    Duration::from_secs_f32(1.5),
                ),
            ));
        }
        RulerSpawnTitle::Big => {
            commands.spawn(TitleInTimer::default());

            let direction = Vec2::from_angle(TITLE_ANGLE);
            let up_direction = Vec2::from_angle(TITLE_ANGLE + std::f32::consts::FRAC_PI_2);
            let up_shift = up_direction * TITLE_SIZE.y - direction * TITLE_SIZE.x * 2.05;
            let rotation = Quat::from_rotation_z(TITLE_ANGLE);

            commands.spawn((
                EntryBigTitle {
                    move_vec: Vec2::ZERO,
                },
                EntryBigTitleMiddle,
                Animation::new("Ruler_BigTitle_Middle", TITLE_SIZE)
                    .with_color(TITLE_COLOR.with_alpha(0.0)),
                Transform::from_translation(vec3(0.0, 0.0, TITLE_Z)).with_rotation(rotation),
            ));
            commands.spawn((
                EntryBigTitle {
                    move_vec: direction * TITLE_SIZE.x * 2.0,
                },
                Animation::new("Ruler_BigTitle_Upper", TITLE_SIZE).with_color(TITLE_COLOR),
                Transform::from_translation(vec3(up_shift.x, up_shift.y, TITLE_Z))
                    .with_rotation(rotation),
            ));
            commands.spawn((
                EntryBigTitle {
                    move_vec: direction * -TITLE_SIZE.x * 2.0,
                },
                Animation::new("Ruler_BigTitle_Lower", TITLE_SIZE).with_color(TITLE_COLOR),
                Transform::from_translation(vec3(-up_shift.x, -up_shift.y, TITLE_Z))
                    .with_rotation(rotation),
            ));
            commands.spawn(Sound::new("Ruler_Intro"));
        }
    }
}

fn ruler_handle_title_movements(
    mut commands: Commands,
    mut timer: Single<(Entity, &mut TitleInTimer)>,
    time: Res<Time>,
    mut q_title: Query<(
        &mut Transform,
        &mut Sprite,
        &EntryBigTitle,
        Option<&EntryBigTitleMiddle>,
    )>,
) {
    let (ref entity, ref mut timer) = *timer;
    if timer.tick(time.delta()).just_finished() {
        commands.entity(*entity).despawn();
        commands.spawn(TitleOutTimer::default());
        return;
    }
    let fraction = timer.fraction();
    let delta = QuadraticOutCurve.sample(fraction).unwrap()
        - QuadraticOutCurve
            .sample((fraction - time.delta_secs() / TITLE_IN_TIME).max(0.0))
            .unwrap();
    for (mut transform, mut sprite, big_title, middle) in q_title.iter_mut() {
        if middle.is_some() {
            sprite.color.set_alpha(fraction);
        } else {
            let displace = delta * big_title.move_vec;
            transform.translation.x += displace.x;
            transform.translation.y += displace.y;
        }
    }
}

fn ruler_handle_title_fade_out(
    mut timer: Single<(Entity, &mut TitleOutTimer)>,
    mut commands: Commands,
    mut q_title: Query<&mut Sprite, With<EntryBigTitle>>,
    q_title_entities: Query<Entity, With<EntryBigTitle>>,
    time: Res<Time>,
) {
    let (ref entity, ref mut timer) = *timer;
    if timer.tick(time.delta()).just_finished() {
        commands.entity(*entity).despawn();
        for entity in q_title_entities.iter() {
            commands.entity(entity).despawn();
        }
        commands.trigger(RulerStart);
        return;
    }
    let fraction = timer.fraction();
    for mut sprite in q_title.iter_mut() {
        sprite.color.set_alpha(1.0 - fraction);
    }
}

fn ruler_start(_event: On<RulerStart>, mut commands: Commands) {
    commands.spawn((Music, Sound::new("Ruler_Bgm")));
}
