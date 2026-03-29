use crate::prelude::*;
use canum_res::Animation;
use canum_res::background::Background;

pub(super) struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(choose_background);
    }
}

fn choose_background(
    event: On<canum_play::setup::PostStartSession>,
    mut commands: Commands,
    q_background: Query<Entity, With<Background>>,
) {
    for entity in q_background.iter() {
        commands.entity(entity).despawn();
    }
    let fullscreen_size = Vec2::new(
        CONFIG.display.virtual_size.0 as f32,
        CONFIG.display.virtual_size.1 as f32,
    );
    match event.fight.as_str() {
        "Apple" => {
            commands.spawn((
                Background::new(fullscreen_size),
                Animation::new("Apple_TreeEmpty", fullscreen_size)
                    .with_color(Color::default().with_alpha(0.5)),
            ));
        }
        _ => {}
    }
}
