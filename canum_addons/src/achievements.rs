use crate::prelude::*;

pub(super) struct AchievementsPlugin;

impl Plugin for AchievementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(get_achievement)
            .add_observer(show_achievement);
        app.add_systems(FixedPreUpdate, fade_achievement);
    }
}

#[derive(Event, Debug, Clone)]
pub struct GetAchievement {
    pub name: String,
}

fn get_achievement(
    event: On<GetAchievement>,
    mut save: ResMut<Save>,
    mut commands: Commands,
    lang: Res<Lang>,
) {
    if save.progress.achievements.insert(&event.name) {
        let details = CONFIG.values.achievement.get(&event.name);
        let kind = details.map(|details| details.kind).unwrap_or_default();
        let title = lang.get(&format!("Achievement_{}_Title", event.name));
        let picture = format!("Achievement_{}", event.name);
        commands.trigger(ShowAchievement {
            name: event.name.clone(),
            picture,
            title: title.to_owned(),
            kind,
        });
        commands.spawn(Sound::new(format!("Ui_Achievement_{}", kind.as_name())));
    }
}

const MAX_TRANSPARENCY: f32 = 1.0;
const MIN_TRANSPARENCY: f32 = 0.0;
const FADE_TIME: f32 = 0.5;
const HOLD_OPAQUE_TIME: f32 = 2.0;

#[derive(Event, Debug, Clone)]
struct ShowAchievement {
    name: String,
    picture: String,
    title: String,
    kind: canum_res::config::AchievementKind,
}

#[derive(Component, Debug)]
#[require(Node)]
struct AchievementPopup {
    timer: Timer,
}
impl Default for AchievementPopup {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(HOLD_OPAQUE_TIME + FADE_TIME * 2.0, TimerMode::Once),
        }
    }
}

fn show_achievement(
    event: On<ShowAchievement>,
    mut commands: Commands,
    q_bottom_right: Query<Entity, With<canum_ui::BottomRight>>,
    fonts: Res<canum_ui::Fonts>,
) {
    let Ok(bottom_right) = q_bottom_right.single() else {
        return;
    };
    info!("Showing achievement {}", event.name);
    const WIDTH: f32 = 200.0;
    const HEIGHT: f32 = 150.0;
    const IMAGE_WIDTH: f32 = 32.0;
    const IMAGE_START_Y: f32 = 5.0 - HEIGHT * 0.5 + IMAGE_WIDTH * 0.5;
    commands.spawn((
        ChildOf(bottom_right),
        AchievementPopup::default(),
        Node {
            margin: UiRect::all(Val::Auto),
            width: px(WIDTH),
            height: px(HEIGHT),
            ..default()
        },
        canum_ui::Animation::new(
            format!("Achievement_Background_{}", event.kind.as_name()),
            Vec2::new(WIDTH, HEIGHT),
        )
        .with_color(Color::Srgba(Srgba::WHITE.with_alpha(MIN_TRANSPARENCY))),
        children![
            (
                Node {
                    position_type: PositionType::Absolute,
                    top: px(IMAGE_START_Y),
                    width: px(IMAGE_WIDTH),
                    height: px(IMAGE_WIDTH),
                    margin: UiRect::all(Val::Auto),
                    ..default()
                },
                canum_ui::Animation::new(event.picture.clone(), vec2(IMAGE_WIDTH, IMAGE_WIDTH))
                    .with_color(Color::Srgba(Srgba::WHITE.with_alpha(MIN_TRANSPARENCY))),
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    width: px(WIDTH - 5.0),
                    top: px(IMAGE_START_Y + 32.0),
                    margin: UiRect::all(Val::Auto),
                    ..default()
                },
                Text::new(event.title.clone()),
                TextLayout {
                    linebreak: LineBreak::WordOrCharacter,
                    ..default()
                },
                TextFont {
                    font: fonts.desc.clone(),
                    font_size: 16.0,
                    font_smoothing: bevy::text::FontSmoothing::None,
                    ..default()
                },
                TextColor(Color::Srgba(Srgba::WHITE.with_alpha(MIN_TRANSPARENCY))),
            )
        ],
    ));
}

fn fade_achievement(
    mut q_achievement: Query<(Entity, &mut ImageNode, &mut AchievementPopup, &Children)>,
    mut q_changeable: Query<
        (Option<&mut ImageNode>, Option<&mut TextColor>),
        Without<AchievementPopup>,
    >,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut image_node, mut popup, children) in q_achievement.iter_mut() {
        if popup.timer.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
        } else {
            let elapsed = popup.timer.elapsed_secs();
            let alpha = if elapsed <= FADE_TIME {
                let percentage = elapsed / FADE_TIME;
                percentage * (MAX_TRANSPARENCY - MIN_TRANSPARENCY) + MIN_TRANSPARENCY
            } else if elapsed <= FADE_TIME + HOLD_OPAQUE_TIME {
                MAX_TRANSPARENCY
            } else {
                let percentage = (elapsed - HOLD_OPAQUE_TIME - FADE_TIME) / FADE_TIME;
                percentage * (MIN_TRANSPARENCY - MAX_TRANSPARENCY) + MAX_TRANSPARENCY
            };
            let alpha = alpha.clamp(0.0, 1.0);
            image_node.color.set_alpha(alpha);
            for child in children.iter() {
                let Ok((image_node, text_color)) = q_changeable.get_mut(child) else {
                    return;
                };
                if let Some(mut image_node) = image_node {
                    image_node.color.set_alpha(alpha);
                }
                if let Some(mut text_color) = text_color {
                    text_color.0.set_alpha(alpha);
                }
            }
        }
    }
}
