use crate::prelude::*;
use std::{sync::Mutex, time::Duration};

pub(super) struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (modify_animation, tick_animation));
    }
}

#[derive(Debug, Clone, Component)]
#[require(AnimationClock, ImageNode)]
pub struct Animation {
    pub name: String,
    pub size: Vec2,
    pub repeating: bool,
    /// Pause the animation on a certain frame.
    pub pause: Option<usize>,
}
#[derive(Default, Debug, Component)]
pub(crate) struct AnimationClock {
    timer: Timer,
    total: usize,
}

impl Animation {
    /// Creates an animation with given size.
    pub fn new(name: impl Into<String>, size: Vec2) -> Self {
        Self {
            name: name.into(),
            size,
            repeating: false,
            pause: None,
        }
    }
    /// Sets the animation to repeating mode.
    pub fn with_repeating(mut self) -> Self {
        self.repeating = true;
        self
    }
    /// Pauses the animation on the given frame index.
    pub fn with_pause(mut self, index: usize) -> Self {
        self.pause = Some(index);
        self
    }
}
impl Default for Animation {
    fn default() -> Self {
        Self::new("Empty", Vec2::new(32.0, 32.0))
    }
}

fn convert_to_image_node(
    image_node: &mut ImageNode,
    name: String,
    asset_server: &AssetServer,
    atlas: &canum_res::config::SpriteAtlas,
    layouts: &mut Assets<TextureAtlasLayout>,
    atlas_handles: &mut canum_res::AnimationAtlasHandles,
    image_handles: &mut canum_res::AnimationImageHandles,
) {
    let image = image_handles
        .entry(name.clone())
        .or_insert_with(|| asset_server.load(atlas.path.clone()))
        .clone();
    let layout = atlas_handles
        .entry(name)
        .or_insert_with(|| {
            layouts.add(TextureAtlasLayout::from_grid(
                UVec2 {
                    x: atlas.size.0,
                    y: atlas.size.1,
                },
                atlas.count,
                1,
                None,
                Some(UVec2 {
                    x: atlas.offset.0,
                    y: atlas.offset.1,
                }),
            ))
        })
        .clone();
    image_node.image = image;
    image_node.texture_atlas = Some(TextureAtlas { layout, index: 0 });
}

fn modify_animation(
    mut query: Query<
        (
            &Animation,
            &mut ImageNode,
            &mut AnimationClock,
            &mut Visibility,
        ),
        Changed<Animation>,
    >,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlas_handles: ResMut<canum_res::AnimationAtlasHandles>,
    mut image_handles: ResMut<canum_res::AnimationImageHandles>,
) {
    let default_image = CONFIG
        .assets
        .sprites
        .get("Empty")
        .and_then(|sprites| sprites.first())
        .map(|sprite| {
            let mut image_node = ImageNode::default();
            convert_to_image_node(
                &mut image_node,
                "Empty".to_owned(),
                &asset_server,
                sprite,
                &mut layouts,
                &mut atlas_handles,
                &mut image_handles,
            );
            image_node
        })
        .unwrap_or_default();
    let mutex = Mutex::new((layouts, atlas_handles, image_handles));
    query
        .par_iter_mut()
        .for_each(|(animation, mut image_node, mut clock, mut visibility)| {
            if animation.name.is_empty() {
                *visibility = Visibility::Hidden;
            } else {
                *visibility = Visibility::Inherited;
            }
            let Some(config) = CONFIG.assets.sprites.get(&animation.name) else {
                *image_node = default_image.clone();
                return;
            };
            if config.is_empty() {
                *image_node = default_image.clone();
                return;
            }
            let atlas = &config[rand::random_range(0..config.len())];
            {
                let mut guard = mutex.lock().unwrap();
                let (layouts, atlas_handles, image_handles) = &mut *guard;
                convert_to_image_node(
                    &mut image_node,
                    animation.name.clone(),
                    &asset_server,
                    atlas,
                    layouts,
                    atlas_handles,
                    image_handles,
                );
            }
            if let Some(pause) = animation.pause {
                clock.timer = Timer::default();
                clock.timer.finish();
                if let Some(ref mut texture_atlas) = image_node.texture_atlas {
                    texture_atlas.index = pause;
                }
            } else {
                clock.timer = Timer::new(
                    Duration::from_millis(atlas.interval as u64),
                    TimerMode::Repeating,
                );
                clock.total = atlas.count as usize;
            }
        });
}

/// Notifies itself that the animation plays one loop.
#[derive(EntityEvent)]
pub struct AnimationComplete {
    pub entity: Entity,
}

fn tick_animation(
    commands: ParallelCommands,
    mut query: Query<(Entity, &Animation, &mut ImageNode, &mut AnimationClock)>,
    time: Res<Time>,
) {
    query
        .par_iter_mut()
        .for_each(|(entity, animation, mut image_node, mut clock)| {
            if !clock.timer.duration().is_zero() && clock.timer.tick(time.delta()).just_finished() {
                let Some(texture_atlas) = &mut image_node.texture_atlas else {
                    return;
                };
                let next_index = (texture_atlas.index + 1) % clock.total;
                if animation.repeating || next_index != 0 {
                    texture_atlas.index = next_index;
                    commands.command_scope(|mut commands| {
                        commands.trigger(AnimationComplete { entity });
                    });
                } else {
                    // Prevents the timer from generating more ticks, unless the animation is refreshed.
                    clock.timer.set_mode(TimerMode::Once);
                    clock.timer.finish();
                }
            }
        });
}
