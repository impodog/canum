use std::time::Duration;

use crate::prelude::*;

pub(super) struct TransitionPlugin;

impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, (init_pure_color, work_pure_color).chain());
    }
}

/// Marks this entity to do transition to pure color background and then show up.
#[derive(Component, Debug, Clone)]
#[require(Transform, Visibility::Hidden, PureColorClock)]
pub struct PureColor {
    pub destroy: Option<Entity>,
    pub duration: Duration,
    pub color: Color,
    pub remove_self: bool,
}

/// Notifies itself that the pure color has played through half point.
#[derive(EntityEvent, Debug)]
pub struct PureColorHalfPoint {
    pub entity: Entity,
}

#[derive(Component, Debug, Clone, Default, Deref, DerefMut)]
struct PureColorClock {
    #[deref]
    timer: Timer,
    half_timer: Timer,
    child: Option<Entity>,
}

fn init_pure_color(
    mut commands: Commands,
    mut q_pure_color: Query<(Entity, &mut PureColor), Added<PureColor>>,
) {
    for (entity, mut pure_color) in q_pure_color.iter_mut() {
        pure_color.color = pure_color.color.to_linear().into();
        let child = commands
            .spawn((
                ChildOf(entity),
                Transform::from_translation(Vec3::new(0.0, 0.0, 114.37)),
                Sprite {
                    custom_size: Some(Vec2::new(
                        CONFIG.display.virtual_size.0 as f32,
                        CONFIG.display.virtual_size.1 as f32,
                    )),
                    color: pure_color.color.with_alpha(0.0),
                    ..default()
                },
                Visibility::Visible,
            ))
            .id();
        commands.entity(entity).insert(PureColorClock {
            timer: Timer::new(pure_color.duration, TimerMode::Once),
            half_timer: Timer::new(pure_color.duration.mul_f64(0.5), TimerMode::Once),
            child: Some(child),
        });
    }
}
fn work_pure_color(
    mut commands: Commands,
    mut q_pure_color: Query<(Entity, &PureColor, &mut PureColorClock, &mut Visibility)>,
    mut q_sprite: Query<&mut Sprite>,
    time: Res<Time>,
) {
    for (entity, pure_color, mut clock, mut visibility) in q_pure_color.iter_mut() {
        let Some(child) = clock.child else {
            continue;
        };
        let Ok(mut sprite) = q_sprite.get_mut(child) else {
            continue;
        };
        clock.tick(time.delta());
        clock.half_timer.tick(time.delta());
        let half = pure_color.duration.as_secs_f32() * 0.5;
        let current = clock.elapsed().as_secs_f32();
        if current >= half {
            sprite.color.set_alpha(
                QuadraticOutCurve
                    .sample((half * 2.0 - current) / half)
                    .unwrap()
                    * pure_color.color.alpha(),
            );
        } else {
            sprite.color.set_alpha(
                CubicInOutCurve.sample(current / half).unwrap() * pure_color.color.alpha(),
            );
        }
        if clock.half_timer.just_finished() {
            if let Some(destroy) = pure_color.destroy
                && let Ok(mut commands) = commands.get_entity(destroy)
            {
                commands.despawn();
            }
            *visibility = Visibility::Inherited;
            commands.trigger(PureColorHalfPoint { entity });
        }
        if clock.timer.just_finished() {
            if pure_color.remove_self {
                commands.entity(entity).despawn();
            } else {
                commands.entity(child).despawn();
                commands
                    .entity(entity)
                    .remove::<PureColor>()
                    .remove::<PureColorClock>();
            }
        }
    }
}
