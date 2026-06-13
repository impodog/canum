use crate::config;
use bevy::prelude::*;
use std::{
    collections::HashMap,
    sync::{
        Mutex,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};

#[derive(Debug, Component)]
#[require(AnimationClock, Sprite)]
pub struct Animation {
    pub name: String,
    pub size: Vec2,
    pub scale: Vec2,
    /// Instruct the animation to pause before a certain frame index. Set to 0 to play the animation once.
    pub pause: AtomicUsize,
    pub color: Color,
    pub visibility: Visibility,
    pub inform: Mutex<Option<AnimationInform>>,
    pub starting_index: usize,
    pub self_despawn: bool,
}
impl Clone for Animation {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            size: self.size,
            scale: self.scale,
            pause: AtomicUsize::new(self.pause.load(Ordering::Acquire)),
            color: self.color,
            visibility: self.visibility,
            inform: Mutex::new(self.inform.lock().unwrap().clone()),
            starting_index: 0,
            self_despawn: self.self_despawn,
        }
    }
}

#[derive(Default, Debug, Component)]
pub(crate) struct AnimationClock {
    timer: Timer,
    total: usize,
    paused: bool,
}

#[derive(Debug, Clone)]
pub struct AnimationInform {
    /// The entity to send `AnimationComplete` to.
    pub entity: Entity,
    /// The frame indices to send the event. If the index is 0 or out of bounds, the event will be sent after the last frame.
    pub index: Vec<usize>,
}

impl Animation {
    /// Creates an animation with given size.
    pub fn new(name: impl Into<String>, size: Vec2) -> Self {
        Self {
            name: name.into(),
            size,
            scale: Vec2::new(1.0, 1.0),
            pause: AtomicUsize::new(usize::MAX),
            color: Color::default(),
            visibility: Visibility::default(),
            inform: Mutex::new(None),
            starting_index: 0,
            self_despawn: false,
        }
    }
    /// Creates a once-animation. This sends `self.inform` `AnimationComplete` after complete playing, if any.
    pub fn once(self) -> Self {
        self.pause.store(0, Ordering::Release);
        self
    }
    /// This animation plays once and then despawn itself.
    pub fn once_then_despawn(mut self) -> Self {
        self.self_despawn = true;
        self.once()
    }
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }
    /// Informs this entity with `AnimationComplete` after the animation is played at a specific index.
    pub fn with_inform(self, inform: AnimationInform) -> Self {
        *self.inform.lock().unwrap() = Some(inform);
        self
    }

    pub fn set_pause(&self, position: usize) {
        self.pause.store(position, Ordering::Release)
    }
    pub fn set_inform(&self, inform: AnimationInform) {
        *self.inform.lock().unwrap() = Some(inform);
    }

    /// Replace the animation with another one, with only common changeable fields.
    pub fn replace(
        &mut self,
        name: impl Into<String>,
        once: bool,
        inform: Option<AnimationInform>,
    ) {
        self.name = name.into();
        self.set_pause(if once { 0 } else { usize::MAX });
        *self.inform.lock().unwrap() = inform;
    }
}
impl Default for Animation {
    fn default() -> Self {
        Self::new("Empty", Vec2::new(32.0, 32.0))
    }
}

#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct AnimationAtlasHandles(HashMap<String, Handle<TextureAtlasLayout>>);

#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct AnimationImageHandles(HashMap<String, Handle<Image>>);

#[allow(clippy::too_many_arguments)]
fn convert_to_sprite(
    sprite: &mut Sprite,
    name: String,
    asset_server: &AssetServer,
    atlas_name: String,
    atlas: &crate::config::SpriteAtlas,
    starting_index: usize,
    layouts: &mut Assets<TextureAtlasLayout>,
    atlas_handles: &mut AnimationAtlasHandles,
    image_handles: &mut AnimationImageHandles,
) {
    let image = image_handles
        .entry(name.clone())
        .or_insert_with(|| asset_server.load(atlas.path.clone()))
        .clone();
    sprite.image = image;
    let layout = atlas_handles
        .0
        .entry(atlas_name)
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
    sprite.texture_atlas = Some(TextureAtlas {
        layout,
        index: starting_index,
    });
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
    mut image_handles: ResMut<AnimationImageHandles>,
    commands: ParallelCommands,
) {
    let mut default_sprite = Sprite::default();
    config::CONFIG
        .assets
        .sprites
        .get("Empty")
        .and_then(|sprites| sprites.first())
        .inspect(|sprite_atlas| {
            convert_to_sprite(
                &mut default_sprite,
                "Empty".to_owned(),
                &asset_server,
                "Empty".to_owned(),
                sprite_atlas,
                0,
                &mut layouts,
                &mut atlas_handles,
                &mut image_handles,
            )
        });
    let mutex = Mutex::new((layouts, atlas_handles, image_handles));
    query
        .par_iter_mut()
        .for_each(|(animation, mut sprite, mut clock, mut visibility)| {
            if animation.name.is_empty() {
                *visibility = Visibility::Hidden;
            } else if *visibility != animation.visibility {
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
            let atlas_index = rand::random_range(0..config.len());
            let atlas = &config[atlas_index];
            let atlas_name = format!("{}{atlas_index}", animation.name);

            {
                let mut guard = mutex.lock().unwrap();
                let (layouts, atlas_handles, image_handles) = &mut *guard;
                convert_to_sprite(
                    sprite.as_mut(),
                    animation.name.clone(),
                    &asset_server,
                    atlas_name.clone(),
                    atlas,
                    animation.starting_index,
                    layouts,
                    atlas_handles,
                    image_handles,
                );
            }
            sprite.color = animation.color;
            sprite.custom_size = Some(animation.size * animation.scale);
            clock.timer = Timer::new(
                Duration::from_millis(atlas.interval as u64),
                TimerMode::Repeating,
            );
            clock.total = atlas.count as usize;
            commands.command_scope(|mut commands| {
                commands.trigger(UpdateSpriteHandle(animation.name.clone()));
                commands.trigger(UpdateSpriteHandle(atlas_name));
            });
        });
}

#[derive(EntityEvent, Debug, Clone)]
pub struct AnimationComplete {
    pub entity: Entity,
    pub source: Entity,
    pub index: usize,
}

pub(crate) fn tick_animation(
    commands: ParallelCommands,
    mut query: Query<(Entity, Ref<Animation>, &mut Sprite, &mut AnimationClock)>,
    time: Res<Time>,
) {
    query
        .par_iter_mut()
        .for_each(|(entity, animation, mut sprite, mut clock)| {
            let Some(texture_atlas) = &mut sprite.texture_atlas else {
                return;
            };
            if animation.is_changed() {
                return;
            }
            let pause = animation.pause.load(Ordering::Acquire);
            let just_unpaused = if clock.paused {
                if (texture_atlas.index + 1) % clock.total == pause {
                    return;
                } else {
                    clock.paused = false;
                    clock.timer.reset();
                }
                true
            } else {
                false
            };
            if !clock.timer.duration().is_zero()
                && (just_unpaused || clock.timer.tick(time.delta()).just_finished())
            {
                let next_index = (texture_atlas.index + 1) % clock.total;
                if next_index == pause {
                    // Prevents clock from triggering again.
                    clock.paused = true;
                } else {
                    texture_atlas.index = next_index;
                }
                if let Some(ref inform) = *animation.inform.lock().unwrap() {
                    for index in inform.index.iter().copied() {
                        let after_last = index == 0 || index >= clock.total;
                        if (after_last && next_index == 0) || index == next_index {
                            commands.command_scope(|mut commands| {
                                commands.trigger(AnimationComplete {
                                    entity: inform.entity,
                                    source: entity,
                                    index,
                                });
                            });
                        }
                    }
                }
                if clock.paused && animation.self_despawn {
                    // This is checked after triggering AnimationComplete for further operations, if any.
                    commands.command_scope(|mut commands| {
                        commands.entity(entity).despawn();
                    });
                }
            }
        });
}

#[derive(Deref, DerefMut)]
pub(crate) struct RandomClearingTimer(Timer);
impl Default for RandomClearingTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(30.0, TimerMode::Repeating))
    }
}

/// Stores the last updated time of a named handle.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct UpdateTimes(HashMap<String, AtomicU64>);

/// Refresh this handle's last updated time.
#[derive(Event)]
pub struct UpdateSpriteHandle(pub String);

/// This will create a new entry.
#[derive(Resource, Default)]
pub(crate) struct UpdateHandleStrongQueue(Vec<String>);

pub(crate) fn update_handle(
    event: On<UpdateSpriteHandle>,
    update_time: Res<UpdateTimes>,
    time: Res<Time>,
    mut strong_queue: ResMut<UpdateHandleStrongQueue>,
) {
    if let Some(value) = update_time.get(&event.0) {
        value.store(time.elapsed().as_secs(), Ordering::Release);
    } else {
        strong_queue.0.push(event.0.clone());
    }
}
pub(crate) fn update_handle_strong(
    mut strong_queue: ResMut<UpdateHandleStrongQueue>,
    mut update_time: ResMut<UpdateTimes>,
    time: Res<Time>,
) {
    let tick = time.elapsed().as_secs();
    while let Some(name) = strong_queue.0.pop() {
        update_time.insert(name, AtomicU64::new(tick));
    }
}

/// Scans all stored handles, and randomly clear some to free up space.
pub(crate) fn random_clearing(
    mut image_handles: ResMut<AnimationImageHandles>,
    mut atlas_handles: ResMut<AnimationAtlasHandles>,
    mut timer: Local<RandomClearingTimer>,
    update_times: Res<UpdateTimes>,
    time: Res<Time>,
) {
    let current_tick = time.elapsed().as_secs();
    if timer.tick(time.delta()).is_finished() {
        let new_handles = image_handles
            .0
            .drain()
            .filter(|(name, _)| {
                let Some(update_time) = update_times.0.get(name) else {
                    return true;
                };
                let update_time = update_time.load(Ordering::Acquire);
                current_tick - update_time <= 15
            })
            .collect::<HashMap<_, _>>();
        image_handles.0 = new_handles;
        let new_handles = atlas_handles
            .0
            .drain()
            .filter(|(name, _)| {
                let Some(update_time) = update_times.0.get(name) else {
                    return true;
                };
                let update_time = update_time.load(Ordering::Acquire);
                current_tick - update_time <= 31
            })
            .collect::<HashMap<_, _>>();
        atlas_handles.0 = new_handles;
    }
}
