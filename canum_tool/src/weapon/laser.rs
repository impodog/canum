use bevy::sprite::Anchor;

use crate::prelude::*;
use std::sync::Mutex;

pub(super) struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<LaserLike>()
            .on_add(laser_like_hook);
        app.init_resource::<SpriteUpdateBuffer>();
        app.add_systems(
            FixedUpdate,
            (update_laser_like, (apply_laser_buffer, apply_length_change)).chain(),
        );
        app.add_systems(FixedPreUpdate, add_for_friendly);
        app.add_systems(FixedUpdate, sprite_playback);
        app.register_required_components_with::<projectile::Projectile, _>(|| {
            LaserLayer::LASER_PROJECTILE
        });
        app.register_required_components::<health::Friendly, LaserLayer>();
    }
}

fn add_for_friendly(
    mut q_friendly: Query<(&health::Friendly, &mut LaserLayer), Changed<health::Friendly>>,
) {
    q_friendly
        .par_iter_mut()
        .for_each(|(friendly, mut laser_layer)| {
            if friendly.0 {
                laser_layer.0 =
                    (laser_layer.0 | LaserLayer::LASER_PLAYER.0) & !LaserLayer::LASER_ENEMY.0;
            } else {
                laser_layer.0 =
                    (laser_layer.0 | LaserLayer::LASER_ENEMY.0) & !LaserLayer::LASER_PLAYER.0;
            }
        });
}

/// Creates laser-like visual effects by duplicating animations, and sends touch events.
/// Defaults to shoots to the right if not rotated, but base direction can be changed when initializing.
///
/// You can apply effects such as contact damage by inserting it directly to this entity, since there is just one collider(sensor).
#[derive(Component, Debug, Clone)]
#[require(
    Transform,
    Visibility,
    LaserLikeInfo,
    Collider,
    Sensor,
    CollidingEntities,
    projectile::NoDisposeProjectile
)]
pub struct LaserLike {
    pub base_direction: Dir2,
    pub middle: SpriteSheet,
    pub terminal: SpriteSheet,
    pub playback_interval: f32,
    pub collide_width: f32,
    pub length: f32,
    /// This is not collider width. It is the width of the sprite.
    pub width: f32,
    /// Bitmap to ignore certain entities that has one of these layers.
    pub ignore_layer: LaserLayer,
    pub animation_kind: LaserAnimationKind,
}
impl Default for LaserLike {
    fn default() -> Self {
        Self {
            base_direction: Dir2::new_unchecked(vec2(1.0, 0.0)),
            middle: default(),
            terminal: default(),
            playback_interval: 1.0,
            collide_width: 1.0,
            length: 32.0,
            width: 32.0,
            ignore_layer: LaserLayer(0),
            animation_kind: LaserAnimationKind::Clipped,
        }
    }
}

/// In `LaserLike`, controls how to deal with the fraction part of the animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LaserAnimationKind {
    /// Clip the terminal animation so that the length matches exactly where the hitbox ends.
    #[default]
    Clipped,
    /// Ceil the number of animations to a integer. This may cause animations to be longer than the hitbox.
    Integer,
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
/// Layer that the laser should respect.
pub struct LaserLayer(pub u32);

impl LaserLayer {
    pub const LASER_ENEMY: LaserLayer = LaserLayer(0b1);
    pub const LASER_PLAYER: LaserLayer = LaserLayer(0b10);
    pub const LASER_PROJECTILE: LaserLayer = LaserLayer(0b100);
}
impl std::ops::BitOr for LaserLayer {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[derive(Component, Default)]
struct LaserLikeInfo {
    children: Vec<Entity>,
    total_length: f32,
    total_length_changed: bool,
    playback: Timer,
}

fn laser_like_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let laser = world.get::<LaserLike>(entity).unwrap();
    let base_direction = laser.base_direction;
    let playback_interval = laser.playback_interval;
    world.commands().entity(entity).insert((
        RayCaster::new(vec2(0.0, 0.0), base_direction)
            .with_max_distance(CONFIG.display.screen_size.length())
            .with_solidness(true),
        RayHits::default(),
    ));
    if let Some(mut info) = world.get_mut::<LaserLikeInfo>(entity) {
        info.playback = Timer::from_seconds(playback_interval, TimerMode::Repeating);
    }
}

/// Buffers laser animation updates to avoid blocking animation reads.
/// 0 - The sprite sheet entity.
/// 1 - New sprite sheet.
/// 2 - If Some, the truncated length, or the sprite will be fully shown.
/// 3 - The original size of the sprite.
#[derive(Resource, Default, Debug, Deref, DerefMut)]
struct SpriteUpdateBuffer(Vec<(Entity, SpriteSheet, Option<f32>, Vec2)>);

fn calc_clipped_rect(original_size: Vec2, length: f32) -> Rect {
    Rect::new(
        (original_size.x - length).max(0.0),
        0.0,
        original_size.x,
        original_size.y,
    )
}

fn update_laser_like(
    mut q_laser: Query<(Entity, &LaserLike, &mut LaserLikeInfo, &RayHits)>,
    buffer: ResMut<SpriteUpdateBuffer>,
    q_no_dispose_projectile: Query<&projectile::NoDisposeProjectile>,
    q_layer: Query<&LaserLayer>,
    commands: ParallelCommands,
) {
    let buffer = Mutex::new(buffer);
    q_laser
        .par_iter_mut()
        .for_each(|(entity, laser, mut info, hits)| {
            if let Some(hit) = hits.iter_sorted().find(|hit| {
                q_no_dispose_projectile.get(hit.entity).is_err()
                    && !q_layer
                        .get(hit.entity)
                        .is_ok_and(|layer| layer.0 & laser.ignore_layer.0 != 0)
            }) {
                let target_size = (hit.distance / laser.length).ceil() as usize;
                let current_len = info.children.len();
                let last_length = match laser.animation_kind {
                    LaserAnimationKind::Clipped => Some(hit.distance.rem_euclid(laser.length)),
                    LaserAnimationKind::Integer => None,
                };
                if target_size < current_len {
                    commands.command_scope(|mut commands| {
                        for child in info.children.drain(target_size..current_len) {
                            commands.entity(child).despawn();
                        }
                    });
                    if let Some(last_child) = info.children.last().copied() {
                        buffer.lock().unwrap().push((
                            last_child,
                            laser.terminal.clone(),
                            last_length,
                            vec2(laser.length, laser.width),
                        ));
                    }
                } else if target_size > current_len {
                    if let Some(last_child) = info.children.last().copied() {
                        buffer.lock().unwrap().push((
                            last_child,
                            laser.middle.clone(),
                            None,
                            vec2(laser.length, laser.width),
                        ));
                    }
                    for index in current_len..target_size {
                        let distance = index as f32 * laser.length;
                        let position = distance * laser.base_direction;
                        let child = commands.command_scope(|mut commands| {
                            if index == target_size - 1 {
                                let rect = last_length.map(|last_length| {
                                    calc_clipped_rect(vec2(laser.length, laser.width), last_length)
                                });
                                let custom_size = if let Some(rect) = rect {
                                    rect.size()
                                } else {
                                    vec2(laser.length, laser.width)
                                };
                                commands
                                    .spawn((
                                        ChildOf(entity),
                                        Transform::from_translation(vec3(
                                            position.x, position.y, 0.0,
                                        )),
                                        laser.terminal.clone(),
                                        Anchor::CENTER_LEFT,
                                        Sprite {
                                            custom_size: Some(custom_size),
                                            rect,
                                            ..default()
                                        },
                                    ))
                                    .id()
                            } else {
                                commands
                                    .spawn((
                                        ChildOf(entity),
                                        Transform::from_translation(vec3(
                                            position.x, position.y, 0.0,
                                        )),
                                        Anchor::CENTER_LEFT,
                                        laser.middle.clone(),
                                        Sprite {
                                            custom_size: Some(vec2(laser.length, laser.width)),
                                            ..default()
                                        },
                                    ))
                                    .id()
                            }
                        });
                        info.children.push(child);
                    }
                } else if last_length.is_some() {
                    // When laser kind is Clipped, we need to spend a overhead to always update the terminal child.
                    if let Some(last_child) = info.children.last().copied() {
                        buffer.lock().unwrap().push((
                            last_child,
                            laser.terminal.clone(),
                            last_length,
                            vec2(laser.length, laser.width),
                        ));
                    }
                }
                if target_size != current_len {
                    info.total_length_changed = true;
                    info.total_length = target_size as f32 * laser.length;
                } else {
                    info.total_length_changed = false;
                }
            }
        });
}

fn apply_laser_buffer(
    mut buffer: ResMut<SpriteUpdateBuffer>,
    mut q_sprite: Query<(&mut SpriteSheet, &mut Sprite)>,
) {
    for (entity, target_sprite_sheet, last_length, original_size) in buffer.drain(..) {
        let Ok((mut sprite_sheet, mut sprite)) = q_sprite.get_mut(entity) else {
            return;
        };
        if let Some(last_length) = last_length {
            sprite.rect = Some(calc_clipped_rect(original_size, last_length));
            sprite.custom_size = sprite.rect.map(|rect| rect.size());
        } else if sprite.rect.is_some() {
            sprite.rect = None;
            sprite.custom_size = Some(original_size);
        }
        *sprite_sheet = target_sprite_sheet;
    }
}

fn apply_length_change(mut q_laser: Query<(&LaserLike, &LaserLikeInfo, &mut Collider)>) {
    q_laser
        .par_iter_mut()
        .for_each(|(laser, info, mut collider)| {
            if info.total_length_changed {
                let distance = info.total_length * 0.5;
                let position = distance * laser.base_direction;
                *collider = Collider::compound(vec![(
                    position,
                    laser.base_direction.to_angle(),
                    Collider::rectangle(info.total_length, laser.collide_width),
                )])
            }
        });
}

fn sprite_playback(
    mut q_laser: Query<(&mut LaserLikeInfo, &Children)>,
    q_sprite_sheet: Query<(&SpriteSheetIndex, &SpriteSheetMeta)>,
    time: Res<Time>,
) {
    q_laser.par_iter_mut().for_each(|(mut info, children)| {
        if info.playback.tick(time.delta()).just_finished() {
            for child in children.iter() {
                let Ok((index, meta)) = q_sprite_sheet.get(child) else {
                    continue;
                };
                let Some(meta) = meta.0 else {
                    return;
                };
                let new_index = (index.get() + 1) % meta.count as usize;
                index.set(new_index);
            }
        }
    });
}
