use crate::prelude::*;

pub(super) struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPreUpdate, keyboard_controls);
    }
}

fn keyboard_controls(
    mut commands: Commands,
    primary_player: Option<Res<crate::player::PrimaryPlayer>>,
    save: Res<Save>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let Some(primary_player) = primary_player.map(|player| **player) else {
        return;
    };
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
    if direction.length_squared() > 1e-8 {
        commands.trigger(crate::player::PlayerMove {
            entity: primary_player,
            rot: direction.to_angle(),
            mult: 1.0,
        });
    }
}
