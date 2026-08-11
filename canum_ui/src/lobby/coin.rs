use crate::prelude::*;

pub(super) struct CoinPlugin;

impl Plugin for CoinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(canum_play::setup::PlayState::Lobby), spawn_coin);
    }
}

#[derive(Component, Default)]
pub struct CoinNumber;

pub fn coin(fonts: &crate::Fonts, number: i32) -> impl Bundle {
    let text_font = TextFont {
        font: fonts.title.clone().into(),
        font_size: FontSize::Px(20.0),
        font_smoothing: FontSmoothing::None,
        ..default()
    };
    (
        Node {
            flex_direction: FlexDirection::Row,
            row_gap: px(3.0),
            ..default()
        },
        children![
            (
                Node { ..default() },
                CoinNumber,
                Text::new(number.to_string()),
                TextColor(Color::WHITE),
                text_font.clone(),
                children![(TextSpan::new(" mol"), TextColor(Color::WHITE), text_font)]
            ),
            (
                Node { ..default() },
                Animation::new("Coin", vec2(50.0, 50.0)).with_repeating()
            ),
        ],
    )
}

fn spawn_coin(
    mut commands: Commands,
    q_top_right: Query<Entity, With<crate::TopRight>>,
    save: Res<Save>,
    fonts: Res<crate::Fonts>,
) {
    let Ok(top_right) = q_top_right.single() else {
        return;
    };
    commands.spawn((ChildOf(top_right), coin(&fonts, save.progress.coins)));
}
