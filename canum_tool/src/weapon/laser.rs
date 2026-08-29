use crate::prelude::*;
use std::sync::Mutex;

pub(super) struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<LaserLike>()
            .on_add(laser_like_hook);
        app.init_resource::<AnimationUpdateBuffer>();
        app.add_systems(
            FixedUpdate,
            (update_laser_like, (apply_laser_buffer, apply_length_change)).chain(),
        );
        app.register_required_components_with::<health::EnemyRelated, _>(|| {
            LaserLayer::LASER_ENEMY
        });
        app.register_required_components_with::<health::PlayerRelated, _>(|| {
            LaserLayer::LASER_PLAYER
        });
    }
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
    pub middle: Animation,
    pub terminal: Animation,
    pub collide_width: f32,
    pub length: f32,
    /// Bitmap to ignore certain entities that has one of these layers.
    pub ignore_layer: LaserLayer,
}
impl Default for LaserLike {
    fn default() -> Self {
        Self {
            base_direction: Dir2::new_unchecked(vec2(1.0, 0.0)),
            middle: default(),
            terminal: default(),
            collide_width: 1.0,
            length: 32.0,
            ignore_layer: LaserLayer(0),
        }
    }
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
/// Layer that the laser should respect.
pub struct LaserLayer(pub u32);

impl LaserLayer {
    pub const LASER_ENEMY: LaserLayer = LaserLayer(0b1);
    pub const LASER_PLAYER: LaserLayer = LaserLayer(0b10);
}

#[derive(Component, Default)]
struct LaserLikeInfo {
    children: Vec<Entity>,
    total_length: f32,
    total_length_changed: bool,
}

fn laser_like_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let base_direction = world.get::<LaserLike>(entity).unwrap().base_direction;
    world.commands().entity(entity).insert((
        RayCaster::new(vec2(0.0, 0.0), base_direction)
            .with_max_distance(CONFIG.display.screen_size.length())
            .with_solidness(true),
        RayHits::default(),
    ));
}

/// Buffers laser animation updates to avoid blocking animation reads.
#[derive(Resource, Default, Debug, Deref, DerefMut)]
struct AnimationUpdateBuffer(Vec<(Entity, Animation)>);

fn update_laser_like(
    mut q_laser: Query<(Entity, &LaserLike, &mut LaserLikeInfo, &RayHits)>,
    buffer: ResMut<AnimationUpdateBuffer>,
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
                if target_size < current_len {
                    commands.command_scope(|mut commands| {
                        for child in info.children.drain(target_size..current_len) {
                            commands.entity(child).despawn();
                        }
                    });
                    if let Some(last_child) = info.children.last().copied() {
                        buffer
                            .lock()
                            .unwrap()
                            .push((last_child, laser.terminal.clone()));
                    }
                } else if target_size > current_len {
                    if let Some(last_child) = info.children.last().copied() {
                        buffer
                            .lock()
                            .unwrap()
                            .push((last_child, laser.middle.clone()));
                    }
                    for index in current_len..target_size {
                        let distance = (index as f32 + 0.5) * laser.length;
                        let position = distance * laser.base_direction;
                        let child = commands.command_scope(|mut commands| {
                            commands
                                .spawn((
                                    ChildOf(entity),
                                    Transform::from_translation(vec3(position.x, position.y, 0.0)),
                                    if index == target_size - 1 {
                                        laser.terminal.clone()
                                    } else {
                                        laser.middle.clone()
                                    },
                                ))
                                .id()
                        });
                        info.children.push(child);
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
    mut buffer: ResMut<AnimationUpdateBuffer>,
    mut q_animation: Query<&mut Animation>,
) {
    for (entity, target_animation) in buffer.drain(..) {
        let Ok(mut animation) = q_animation.get_mut(entity) else {
            return;
        };
        *animation = target_animation;
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
