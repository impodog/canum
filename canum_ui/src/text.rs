use std::time::Duration;

use crate::prelude::*;

pub(super) struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_fonts);
        app.add_systems(FixedUpdate, update_title_color);
    }
}

#[derive(Resource, Default, Clone)]
pub struct Fonts {
    pub title: Handle<Font>,
    pub desc: Handle<Font>,
}

fn setup_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Fonts {
        title: asset_server.load("assets/fonts/JetbrainsMono.ttf"),
        desc: asset_server.load("assets/fonts/Crimson.ttf"),
    });
}

#[derive(Component, Default)]
struct PopupTitleClock {
    /// Delay before fade in effect, used for segmented pop-up effects.
    delay: Timer,
    fade_in: Timer,
    fade_out: Timer,
}

pub fn popup_title(font: Handle<Font>, content: &str, fade: Duration) -> impl Bundle {
    fn trim_prefix_test(value: &str, prefix: char) -> (&str, bool) {
        value
            .strip_prefix(prefix)
            .map(|value| (value, true))
            .unwrap_or((value, false))
    }
    let mut child = Vec::new();
    let mut delay_time = Duration::default();
    for value in content.split('\n') {
        let (value, is_large) = trim_prefix_test(value, '+');
        let (value, is_delayed) = trim_prefix_test(value, '/');
        if is_delayed {
            delay_time += Duration::from_millis(300);
        }
        let font_size = if is_large { 90 } else { 60 };
        child.push((
            Node {
                height: px(font_size),
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            PopupTitleClock {
                delay: Timer::new(delay_time, TimerMode::Once),
                fade_in: Timer::new(fade, TimerMode::Once),
                fade_out: Timer::new(fade, TimerMode::Once),
            },
            Text::new(value),
            TextColor(Color::WHITE.with_alpha(0.0)),
            TextFont {
                font: font.clone(),
                font_size: font_size as f32,
                ..Default::default()
            },
        ));
    }
    (
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Children::spawn(child),
    )
}

fn update_title_color(
    commands: ParallelCommands,
    mut q_text: Query<(Entity, &mut TextColor, &mut PopupTitleClock)>,
    time: Res<Time>,
) {
    q_text
        .par_iter_mut()
        .for_each(|(entity, mut color, mut clock)| {
            if !clock.delay.is_finished() {
                clock.delay.tick(time.delta());
                return;
            }
            if !clock.fade_in.is_finished() {
                clock.fade_in.tick(time.delta());
                let ratio =
                    clock.fade_in.elapsed().as_secs_f32() / clock.fade_in.duration().as_secs_f32();
                if let Some(alpha) = CubicInOutCurve.sample(ratio) {
                    color.set_alpha(alpha);
                }
            } else if !clock.fade_out.is_finished() {
                clock.fade_out.tick(time.delta());
                let ratio = clock.fade_out.elapsed().as_secs_f32()
                    / clock.fade_out.duration().as_secs_f32();
                if let Some(alpha) = CircularInCurve.sample(1.0 - ratio) {
                    color.set_alpha(alpha);
                }
            } else {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                });
            }
        });
}
