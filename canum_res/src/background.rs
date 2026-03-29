use crate::config::CONFIG;
use bevy::prelude::*;

pub(super) struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (add_background, update_background_image));
        app.add_systems(PostUpdate, restrict_background_position);
    }
}

/// Marks this entity to be the central background handler.
#[derive(Component, Debug, Default)]
#[require(Sprite, Transform, GlobalTransform, BackgroundInfo, Children)]
pub struct Background {
    pub size: Vec2,
}
impl Background {
    pub fn new(size: Vec2) -> Self {
        Self { size }
    }
}

/// Generated to keep track of background images;
#[derive(Component, Debug, Default)]
struct BackgroundInfo {
    pub images: Vec<Vec<Entity>>,
    pub x: usize,
    pub y: usize,
}

/// Marks sub background image entities, following the background's animations.
#[derive(Component, Debug, Default)]
#[require(Sprite, Transform, GlobalTransform)]
struct BackgroundSubImage;

#[allow(clippy::type_complexity)]
fn add_background(
    mut commands: Commands,
    mut q_background: Query<
        (
            Entity,
            &Sprite,
            &Background,
            &mut BackgroundInfo,
            &Children,
            &mut Transform,
        ),
        Changed<Background>,
    >,
) {
    for (entity, sprite, background, mut info, children, mut transform) in q_background.iter_mut() {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
        transform.translation.z = -24.37;
        let x = (CONFIG.display.virtual_size.0 as f32 / background.size.x).ceil() as usize | 1;
        let y = (CONFIG.display.virtual_size.1 as f32 / background.size.y).ceil() as usize | 1;
        info.images.clear();
        info.images.resize_with(x, || {
            let mut row = Vec::new();
            row.resize(y, entity);
            row
        });
        info.x = x;
        info.y = y;
        let start_pos = Vec3::new(
            (x / 2) as f32 * -background.size.x,
            (y / 2) as f32 * -background.size.y,
            0.0,
        );
        for i in 0..x {
            for j in 0..y {
                if i == x / 2 && j == y / 2 {
                    // The center entity is taken by background parent itself.
                    continue;
                }
                let transform = Transform::from_translation(
                    start_pos
                        + Vec3::new(
                            i as f32 * background.size.x,
                            j as f32 * background.size.y,
                            0.0,
                        ),
                );
                let child = commands
                    .spawn((
                        ChildOf(entity),
                        BackgroundSubImage,
                        sprite.clone(),
                        transform,
                    ))
                    .id();
                info.images[i][j] = child;
            }
        }
    }
}

fn update_background_image(
    q_background: Query<(Entity, &BackgroundInfo, &Sprite), Changed<Sprite>>,
    mut q_sprite: Query<&mut Sprite, Without<BackgroundInfo>>,
) {
    for (entity, info, sprite) in q_background.iter() {
        for children in info.images.iter() {
            for child in children.iter() {
                if *child != entity {
                    let Ok(mut child_sprite) = q_sprite.get_mut(*child) else {
                        continue;
                    };
                    *child_sprite = sprite.clone();
                }
            }
        }
    }
}

fn restrict_background_position(mut q_background: Query<&mut Transform, With<Background>>) {
    for mut transform in q_background.iter_mut() {
        if transform.translation.x > CONFIG.display.half_virtual_size.0 {
            transform.translation.x -= CONFIG.display.half_virtual_size.0;
        }
        if transform.translation.x < -CONFIG.display.half_virtual_size.0 {
            transform.translation.x += CONFIG.display.half_virtual_size.0;
        }
        if transform.translation.y > CONFIG.display.half_virtual_size.1 {
            transform.translation.y -= CONFIG.display.half_virtual_size.1;
        }
        if transform.translation.y < -CONFIG.display.half_virtual_size.1 {
            transform.translation.y += CONFIG.display.half_virtual_size.1;
        }
    }
}
