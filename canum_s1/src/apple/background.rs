use std::time::Duration;

use canum_play::prelude::*;
use canum_res::background::Background;

pub(super) struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(super::APPLE_STATE.clone()), choose_background);
        app.add_systems(
            FixedPreUpdate,
            (trigger_background_changed, change_background_apple_tree)
                .chain()
                .run_if(in_state(super::APPLE_STATE.clone())),
        );
    }
}

fn choose_background(mut commands: Commands, q_background: Query<Entity, With<Background>>) {
    for entity in q_background.iter() {
        commands.entity(entity).despawn();
    }
    commands.spawn((
        Background::new(CONFIG.display.screen_size),
        Animation::new("Apple_TreeEmpty", CONFIG.display.screen_size)
            .with_color(Color::default().with_alpha(0.7)),
    ));
    commands.insert_resource(AppleTreeChanged::default());
    commands.insert_resource(BackgroundChangeEventSent::default());
}

/// Notifies that the background is changed and the fight begins.
#[derive(Event, Default)]
pub(super) struct AppleTreeBackgroundChanged;

#[derive(Resource, Deref, DerefMut, Default)]
struct AppleTreeChanged(bool);
fn change_background_apple_tree(
    mut commands: Commands,
    fight_time: Res<canum_play::setup::FightTime>,
    q_background: Query<Entity, With<Background>>,
    apple_tree_changed: Option<ResMut<AppleTreeChanged>>,
) {
    let Some(mut apple_tree_changed) = apple_tree_changed else {
        return;
    };
    if apple_tree_changed.0 {
        return;
    }
    if fight_time.elapsed() >= Duration::from_secs(2) {
        let Ok(previous) = q_background.single() else {
            return;
        };
        apple_tree_changed.0 = true;
        commands.spawn((
            Background::new(CONFIG.display.screen_size),
            Animation::new("Apple_TreeBoss", CONFIG.display.screen_size)
                .with_color(Color::default().with_alpha(0.6))
                .with_visibility(Visibility::Hidden),
            canum_ui::transition::PureColor {
                destroy: previous,
                color: Color::linear_rgb(0.5, 0.5, 0.5),
                duration: Duration::from_secs_f32(1.0),
            },
        ));
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
struct BackgroundChangeEventSent(bool);
fn trigger_background_changed(
    mut commands: Commands,
    changed: Option<Res<AppleTreeChanged>>,
    event_sent: Option<ResMut<BackgroundChangeEventSent>>,
    q_transition: Query<(), With<canum_ui::transition::PureColor>>,
) {
    let Some(mut event_sent) = event_sent else {
        return;
    };
    if event_sent.0 {
        return;
    }
    if changed.is_some_and(|changed| changed.0) && q_transition.iter().next().is_none() {
        event_sent.0 = true;
        commands.trigger(AppleTreeBackgroundChanged);
    }
}
