//! Creates and manages pre-fight boss information panel.

use crate::prelude::*;
use crate::text::Fonts;

pub(super) struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {}
}

/// Marks and stores constants of a boss panel.
/// Other variables are in separate components.
#[derive(Component, Debug)]
pub struct BossPanel {
    pub name: String,
}

pub fn boss_panel(fonts: impl AsRef<Fonts>, panel: BossPanel) -> impl Bundle {
    let fonts = fonts.as_ref();
    let title = (
        Node {
            justify_content: JustifyContent::Center,
            margin: UiRect::all(Val::Auto),
            ..default()
        },
        Text::new(&panel.name),
        TextFont {
            font: fonts.title.clone(),
            font_size: 50.0,
            ..default()
        },
    );
    // let start_button = (Node { ..default() }, Button);
    (
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            margin: UiRect::all(Val::Auto),
            padding: UiRect::all(px(10.0)),
            width: percent(100),
            height: percent(100),
            ..default()
        },
        panel,
        BackgroundColor(Color::linear_rgba(0.1, 0.1, 0.1, 0.8)),
        BoxShadow::default(),
        children![title,],
    )
}
