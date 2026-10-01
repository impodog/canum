use crate::prelude::*;

pub(super) struct EmphasisPlugin;

impl Plugin for EmphasisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (expand_and_fade_out, leave_trail, update_trail_sprite),
        );
        app.add_observer(leave_trail_setting);
    }
}

/// Creates a sprite that expands itself while fading out, much like a "scream" effect.
/// This is usually used with `Animation`.
///
/// This despawns itself after completely fading out.
#[derive(Component)]
#[require(Transform, Visibility, Sprite)]
#[non_exhaustive]
pub struct ExpandAndFadeOut {
    /// The maximum multiplier to apply.
    pub max_mult: f32,
    /// The duration to fade out.
    pub timer: Timer,
    /// The multiplier that this component has applied. This should always start as 1.0.
    pub current_mult: f32,
}
impl ExpandAndFadeOut {
    pub fn new(max_mult: f32) -> Self {
        Self {
            max_mult,
            ..default()
        }
    }

    /// Sets the fade out time.
    pub fn with_time(mut self, secs: f32) -> Self {
        self.timer
            .set_duration(std::time::Duration::from_secs_f32(secs));
        self
    }
}
impl Default for ExpandAndFadeOut {
    fn default() -> Self {
        Self {
            current_mult: 1.0,
            timer: Timer::new(std::time::Duration::from_secs_f32(0.6), TimerMode::Once),
            max_mult: 2.0,
        }
    }
}

fn expand_and_fade_out(
    mut q_effect: Query<(Entity, &mut ExpandAndFadeOut, &mut Transform, &mut Sprite)>,
    commands: ParallelCommands,
    time: Res<Time>,
) {
    let delta = time.delta();
    q_effect
        .par_iter_mut()
        .for_each(|(entity, mut effect, mut transform, mut sprite)| {
            if effect.timer.tick(delta).just_finished() {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                });
                return;
            }

            let fraction = effect.timer.fraction();
            let current = CubicOutCurve.sample(fraction).unwrap();
            let mult_ratio = (1.0 + current) / effect.current_mult;
            effect.current_mult = 1.0 + current;
            transform.scale.x *= mult_ratio;
            transform.scale.y *= mult_ratio;

            let current_alpha_curve = QuadraticOutCurve.sample(fraction).unwrap();
            sprite.color.set_alpha(1.0 - current_alpha_curve);
        });
}

/// Creates a visual effect for fast motion, leaving behind a trail of motion path.
/// Note that this is disabled by default, and you must enable it before seeing pathes.
#[derive(Component, Debug)]
#[require(Transform, Visibility, Sprite)]
#[non_exhaustive]
pub struct LeaveTrail {
    pub enabled: bool,
    pub interval: Timer,
    pub linger: std::time::Duration,
    pub rotate_color: bool,
    pub pure_color: bool,
}
impl LeaveTrail {
    pub fn new(interval: f32, linger: f32) -> Self {
        Self {
            enabled: false,
            interval: Timer::from_seconds(interval, TimerMode::Repeating),
            linger: std::time::Duration::from_secs_f32(linger),
            rotate_color: false,
            pure_color: false,
        }
    }
    pub fn new_rotate_color(interval: f32, linger: f32) -> Self {
        Self::new(interval, linger).rotate_color()
    }

    /// The color rotates for the lingering images.
    pub fn rotate_color(mut self) -> Self {
        self.rotate_color = true;
        self
    }

    pub fn pure_color(mut self) -> Self {
        self.pure_color = true;
        self
    }

    /// Enable lingering trails. This is disabled by default.
    pub fn enabled(mut self) -> Self {
        self.enabled = true;
        self
    }
}

/// Sets the state of `LeaveTrail`.
#[derive(EntityEvent, Debug)]
pub struct LeaveTrailSetting {
    pub entity: Entity,
    pub enabled: bool,
}
impl LeaveTrailSetting {
    pub fn enable(entity: Entity) -> Self {
        Self {
            entity,
            enabled: true,
        }
    }
    pub fn disable(entity: Entity) -> Self {
        Self {
            entity,
            enabled: false,
        }
    }
}

fn leave_trail_setting(event: On<LeaveTrailSetting>, mut q_effect: Query<&mut LeaveTrail>) {
    let Ok(mut effect) = q_effect.get_mut(event.entity) else {
        return;
    };
    effect.enabled = event.enabled;
}

#[derive(Component, Default)]
#[require(Sprite)]
struct TrailSprite {
    time: Timer,
    original_color: Hsla,
    rotate: bool,
}

fn leave_trail(
    mut q_effect: Query<(&mut LeaveTrail, &GlobalTransform, &Sprite)>,
    time: Res<Time>,
    commands: ParallelCommands,
    images: ResMut<Assets<Image>>,
) {
    use std::sync::Mutex;
    let images = Mutex::new(images);
    q_effect
        .par_iter_mut()
        .for_each(|(mut effect, global_transform, sprite)| {
            if !effect.enabled {
                return;
            }
            if effect.interval.tick(time.delta()).just_finished() {
                let mut transform = global_transform.compute_transform();
                transform.translation.z -= 0.01;
                let mut sprite = sprite.clone();
                if effect.pure_color {
                    let mut images = images.lock().unwrap();
                    if let Some(image) = images.get(sprite.image.id())
                        && let Some(new_image) = crate::visual::pure_image(image, [255, 255, 255])
                    {
                        let new_handle = images.add(new_image);
                        sprite.image = new_handle;
                    } else {
                        warn!("Unable to create pure color leave-trail sprite");
                    }
                }
                let original_color = if effect.rotate_color {
                    Hsla::from(sprite.color)
                        .with_saturation(1.0)
                        .with_lightness(0.75)
                } else {
                    sprite.color.into()
                };
                commands.command_scope(|mut commands| {
                    commands.spawn((
                        sprite.clone(),
                        transform,
                        TrailSprite {
                            time: Timer::new(effect.linger, TimerMode::Once),
                            original_color,
                            rotate: effect.rotate_color,
                        },
                    ));
                });
            }
        });
}

fn update_trail_sprite(
    mut q_sprite: Query<(Entity, &mut Sprite, &mut TrailSprite, &mut Transform)>,
    time: Res<Time>,
    commands: ParallelCommands,
) {
    use bevy::math::FloatPow;

    q_sprite
        .par_iter_mut()
        .for_each(|(entity, mut sprite, mut trail, mut transform)| {
            if trail.time.tick(time.delta()).just_finished() {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                });
            } else {
                let fraction = trail.time.fraction();
                transform.translation.z -=
                    time.delta_secs() / trail.time.duration().as_secs_f32() * 0.1;
                let mut new_color = trail
                    .original_color
                    .with_alpha(trail.original_color.alpha * (1.0 - fraction).squared());
                if trail.rotate {
                    new_color = new_color.rotate_hue(360.0 * fraction);
                }
                sprite.color = Color::LinearRgba(Color::Hsla(new_color).to_linear());
            }
        });
}
