use crate::prelude::*;

pub(super) struct BarPlugin;

impl Plugin for BarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPostUpdate,
            (update_associated_health_bar, update_health_bar).chain(),
        );
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct HealthBar {
    pub total: f32,
    pub current: f32,
    pub width: f32,
    pub height: f32,
}
impl Default for HealthBar {
    fn default() -> Self {
        Self {
            total: 1.0,
            current: 1.0,
            width: 600.0,
            height: 10.0,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[require(HealthBar)]
pub struct AssociatedBoss(pub Entity);

#[derive(Component, Default)]
struct HealthBarForegroundChild;

pub fn health_bar(fore: Color, back: Color, bar: HealthBar) -> impl Bundle {
    let back_linear = back.to_linear();
    let inverse_back = Color::linear_rgba(
        1.0 - back_linear.red,
        1.0 - back_linear.green,
        1.0 - back_linear.blue,
        back_linear.alpha,
    );
    (
        Node {
            width: px(bar.width),
            height: px(bar.height),
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
                BackgroundColor(inverse_back),
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    width: px(bar.width - 2.0),
                    height: px(bar.height - 2.0),
                    left: px(1.0),
                    bottom: px(1.0),
                    padding: UiRect::all(px(0)),
                    ..default()
                },
                BackgroundColor(back),
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    width: px((bar.current / bar.total) * (bar.width - 2.0)),
                    height: px(bar.height - 2.0),
                    left: px(1.0),
                    bottom: px(1.0),
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
                    node.width = px((bar.current / bar.total) * (bar.width - 2.0));
                }
            }
        }
    }
}

fn update_associated_health_bar(
    mut q_bar: Query<(Entity, &AssociatedBoss, &mut HealthBar)>,
    q_health: Query<&canum_play::enemy::health::EnemyHealth>,
    mut commands: Commands,
) {
    for (entity, associate, mut bar) in q_bar.iter_mut() {
        if let Ok(health) = q_health.get(associate.0) {
            let value = health.value as f32;
            if bar.current != value {
                bar.current = value;
            }
        } else {
            commands.entity(entity).try_despawn();
        }
    }
}
