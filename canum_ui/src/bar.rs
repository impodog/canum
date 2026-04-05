use crate::prelude::*;

pub(super) struct BarPlugin;

impl Plugin for BarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, update_health_bar);
    }
}

#[derive(Component, Debug, Default)]
pub struct HealthBar {
    pub total: f32,
    pub current: f32,
}
impl HealthBar {
    pub const WIDTH: f32 = 600.0;
    pub const HEIGHT: f32 = 10.0;
}

#[derive(Component, Default)]
struct HealthBarForegroundChild;

pub fn health_bar(fore: Color, back: Color, bar: HealthBar) -> impl Bundle {
    (
        Node {
            width: px(HealthBar::WIDTH),
            height: px(HealthBar::HEIGHT),
            left: px(0),
            bottom: px(0),
            ..default()
        },
        bar,
        children![
            (
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    left: px(0),
                    bottom: px(0),
                    padding: UiRect::all(px(0)),
                    ..default()
                },
                BackgroundColor(back),
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    left: px(0),
                    bottom: px(0),
                    padding: UiRect::all(px(0)),
                    ..default()
                },
                BackgroundColor(fore),
                HealthBarForegroundChild,
            ),
        ],
    )
}

fn update_health_bar(
    q_bar: Query<(Ref<HealthBar>, &Children)>,
    mut q_node: Query<&mut Node, With<HealthBarForegroundChild>>,
) {
    for (bar, children) in q_bar.iter() {
        if bar.is_changed() {
            for child in children.iter() {
                if let Ok(mut node) = q_node.get_mut(child) {
                    node.width = percent(bar.current / bar.total * 100.0);
                }
            }
        }
    }
}
