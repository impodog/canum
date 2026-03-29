use crate::config;
use bevy::prelude::*;
use std::{collections::HashMap, time::Duration};

#[derive(Debug, Clone, Component)]
#[require(AnimationClock, Sprite)]
pub struct Animation {
    pub name: String,
    pub size: Vec2,
    pub scale: Vec2,
    pub once: bool,
    pub color: Color,
    pub visibility: Visibility,
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
            scale: Vec2::new(1.0, 1.0),
            once: false,
            color: Color::default(),
            visibility: Visibility::default(),
        }
    }
    /// Creates a once-animation. This sends itself `AnimationComplete` after complete playing.
    pub fn once(mut self) -> Self {
        self.once = true;
        self
    }
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }
}
impl Default for Animation {
    fn default() -> Self {
        Self::new("Empty", Vec2::new(32.0, 32.0))
    }
}

#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct AnimationAtlasHandles(HashMap<String, Handle<TextureAtlasLayout>>);

fn convert_to_sprite(
    name: String,
    asset_server: &AssetServer,
    atlas: &crate::config::SpriteAtlas,
    layouts: &mut Assets<TextureAtlasLayout>,
    atlas_handles: &mut AnimationAtlasHandles,
) -> Sprite {
    let image: Handle<Image> = asset_server.load(atlas.path.clone());
    let layout = atlas_handles
        .0
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
    Sprite {
        image,
        texture_atlas: Some(TextureAtlas { layout, index: 0 }),
        ..Default::default()
    }
}

pub(crate) fn modify_animation(
    mut query: Query<
        (
            &Animation,
            &mut Sprite,
            &mut AnimationClock,
            &mut Visibility,
        ),
        Changed<Animation>,
    >,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlas_handles: ResMut<AnimationAtlasHandles>,
) {
    let default_sprite = config::CONFIG
        .assets
        .sprites
        .get("Empty")
        .and_then(|sprites| sprites.first())
        .map(|sprite| {
            convert_to_sprite(
                "Empty".to_owned(),
                &asset_server,
                sprite,
                &mut layouts,
                &mut atlas_handles,
            )
        })
        .unwrap_or_default();
    query
        .iter_mut()
        .for_each(|(animation, mut sprite, mut clock, mut visibility)| {
            if animation.name.is_empty() {
                *visibility = Visibility::Hidden;
            } else {
                *visibility = animation.visibility;
            }
            let Some(config) = config::CONFIG.assets.sprites.get(&animation.name) else {
                *sprite = default_sprite.clone();
                return;
            };
            if config.is_empty() {
                *sprite = default_sprite.clone();
                return;
            }
            let atlas = &config[rand::random_range(0..config.len())];
            *sprite = convert_to_sprite(
                animation.name.clone(),
                &asset_server,
                atlas,
                &mut layouts,
                &mut atlas_handles,
            );
            sprite.color = animation.color;
            sprite.custom_size = Some(animation.size * animation.scale);
            clock.timer = Timer::new(
                Duration::from_millis(atlas.interval as u64),
                TimerMode::Repeating,
            );
            clock.total = atlas.count as usize;
        });
}

#[derive(EntityEvent, Deref, DerefMut)]
pub struct AnimationComplete(pub Entity);

pub(crate) fn tick_animation(
    commands: ParallelCommands,
    mut query: Query<(Entity, &Animation, &mut Sprite, &mut AnimationClock)>,
    time: Res<Time>,
) {
    query
        .par_iter_mut()
        .for_each(|(entity, animation, mut sprite, mut clock)| {
            if !clock.timer.duration().is_zero() && clock.timer.tick(time.delta()).just_finished() {
                let Some(texture_atlas) = &mut sprite.texture_atlas else {
                    return;
                };
                let next_index = (texture_atlas.index + 1) % clock.total;
                if next_index != 0 || !animation.once {
                    texture_atlas.index = next_index;
                } else {
                    commands.command_scope(|mut commands| {
                        commands.trigger(AnimationComplete(entity));
                    });
                    // Prevents clock from triggering again.
                    clock.timer = Timer::default();
                    clock.timer.finish();
                }
            }
        });
}
