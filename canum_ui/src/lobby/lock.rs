use crate::prelude::*;

pub(super) struct LockPlugin;

impl Plugin for LockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(canum_play::setup::PlayState::Lobby),
            (spawn_top_text,),
        );
        app.add_systems(
            FixedUpdate,
            adjust_top_text.run_if(in_state(canum_play::setup::PlayState::Lobby)),
        );
    }
}

#[derive(Component, Default)]
#[require(Node, Text)]
pub struct LobbyTopText {
    pub prev_handle: String,
}

fn spawn_top_text(
    mut commands: Commands,
    font: Res<crate::Fonts>,
    q_top_center: Query<Entity, With<crate::TopCenter>>,
) {
    let Ok(top_center) = q_top_center.single() else {
        return;
    };
    commands.spawn((
        ChildOf(top_center),
        LobbyTopText::default(),
        TextColor::WHITE,
        TextFont {
            font: font.desc.clone(),
            font_size: 30.0,
            ..default()
        },
    ));
}

fn adjust_top_text(
    mut q_text: Query<(&mut Text, &mut TextColor, &mut LobbyTopText)>,
    fight: Res<State<canum_play::setup::Fight>>,
    q_player: Query<&GlobalTransform, With<canum_play::player::Player>>,
    lang: Res<canum_save::Lang>,
) {
    let Ok((mut text, mut text_color, mut lobby_top_text)) = q_text.single_mut() else {
        return;
    };
    if let Some(stage_details) = CONFIG.values.stage.get(fight.as_str()) {
        let mut opacity: f32 = 0.0;
        let mut notify_text = None;
        for lock in stage_details.locks.iter() {
            for player_transform in q_player.iter() {
                let position = player_transform.translation().xy();
                let current_opacity =
                    1.0 - ((position.x - lock.position).abs() * 0.01).clamp(0.0, 1.0);
                if current_opacity > opacity {
                    notify_text = lock.notify_text.as_ref();
                    opacity = current_opacity;
                }
            }
        }
        if let Some(notify_text) = notify_text
            && opacity > 0.0
        {
            if lobby_top_text.prev_handle != *notify_text {
                text.0 = lang.get_special(notify_text);
                lobby_top_text.prev_handle = notify_text.to_owned();
            }
            text_color.set_alpha(opacity);
        }
        if opacity.abs() < 1e-3 && !text.0.is_empty() {
            text.0.clear();
            lobby_top_text.prev_handle.clear();
        }
    }
}
