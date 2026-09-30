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
    pub linger: f32,
    pub width: f32,
    pub height: f32,
}
impl Default for HealthBar {
    fn default() -> Self {
        Self {
            total: 1.0,
            current: 1.0,
            linger: 0.0,
            width: 600.0,
            height: 10.0,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[require(HealthBar, AssociatedBossLingerTimeout)]
pub struct AssociatedBoss(pub Entity);

#[derive(Component, Deref, DerefMut)]
struct AssociatedBossLingerTimeout {
    #[deref]
    timeout: Timer,
}
impl Default for AssociatedBossLingerTimeout {
    fn default() -> Self {
        let mut timeout = Timer::from_seconds(1.75, TimerMode::Once);
        timeout.finish();
        Self { timeout }
    }
}

#[derive(Component, Default)]
struct HealthBarForegroundChild;

#[derive(Component, Default)]
struct HealthBarLingerChild;

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
                    width: px((bar.linger / bar.total) * (bar.width - 2.0)),
                    height: px(bar.height - 2.0),
                    left: px(1.0),
                    bottom: px(1.0),
                    padding: UiRect::all(px(0)),
                    ..default()
                },
                BackgroundColor(fore.with_alpha(0.2)),
                HealthBarLingerChild,
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
    mut q_fore: Query<
        &mut Node,
        (
            With<HealthBarForegroundChild>,
            Without<HealthBarLingerChild>,
        ),
    >,
    mut q_linger: Query<
        &mut Node,
        (
            With<HealthBarLingerChild>,
            Without<HealthBarForegroundChild>,
        ),
    >,
) {
    for (bar, children) in q_bar.iter() {
        if bar.is_changed() {
            for child in children.iter() {
                if let Ok(mut node) = q_fore.get_mut(child) {
                    node.width = px((bar.current / bar.total) * (bar.width - 2.0));
                } else if let Ok(mut node) = q_linger.get_mut(child) {
                    node.width = px((bar.linger / bar.total) * (bar.width - 2.0));
                }
            }
        }
    }
}

fn update_associated_health_bar(
    mut q_bar: Query<(
        Entity,
        &AssociatedBoss,
        &mut AssociatedBossLingerTimeout,
        &mut HealthBar,
    )>,
    q_health: Query<&canum_play::enemy::health::EnemyHealth>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, associate, mut timeout, mut bar) in q_bar.iter_mut() {
        if let Ok(health) = q_health.get(associate.0) {
            let value = health.value as f32;
            timeout.tick(time.delta());
            if bar.linger < value {
                bar.linger = value;
            }
            if bar.current != value {
                bar.current = value;
                timeout.reset();
            } else if timeout.is_finished() {
                let diff = value - bar.linger;
                let add = diff.signum() * (0.4f32 * bar.total * time.delta_secs()).min(diff.abs());
                bar.linger += add;
            }
        } else {
            commands.entity(entity).try_despawn();
        }
    }
}
