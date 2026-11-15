use bevy::ecs::change_detection::Tick;

use crate::prelude::*;
use canum_save::util::BTreeMultiSet;

pub mod fade;

pub(super) struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(fade::FadePlugin);
        app.add_systems(
            PostUpdate,
            (init_sprite_sheet_player, queue_sprite_sheet).in_set(AnimationSystems::Init),
        );
        app.add_systems(
            PostUpdate,
            (sprite_sheet_player,).in_set(AnimationSystems::Update),
        );
        app.configure_sets(
            PostUpdate,
            (
                AnimationSystems::Init,
                AnimationSystems::Update,
                AnimationSystems::Final,
                canum_res::BaseAnimationSystems,
            )
                .chain(),
        );
        app.add_observer(update_sprite_sheet_queue);
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationSystems {
    Init,
    Update,
    Final,
}

/// Auto advances the sprite sheet index. You can override the interval if needed.
#[derive(Component, Debug)]
#[require(SpriteSheetPlayerTimer)]
pub struct SpriteSheetPlayer {
    pub interval: Option<Duration>,
    pub repeat: bool,
}
impl Default for SpriteSheetPlayer {
    fn default() -> Self {
        Self {
            interval: None,
            repeat: true,
        }
    }
}
impl SpriteSheetPlayer {
    /// The player ends after the last frame.
    pub fn once() -> Self {
        Self {
            interval: None,
            repeat: false,
        }
    }
}
#[derive(Component, Default, Deref, DerefMut, Debug)]
struct SpriteSheetPlayerTimer {
    #[deref]
    timer: Timer,
    initialized: bool,
}

fn init_sprite_sheet_player(
    mut player: Query<(
        Ref<SpriteSheetPlayer>,
        Ref<SpriteSheetMeta>,
        &mut SpriteSheetPlayerTimer,
    )>,
) {
    player.par_iter_mut().for_each(|(player, meta, mut timer)| {
        let player_tick = player.last_changed().get();
        let meta_tick = meta.last_changed().get();
        let latest = player_tick.max(meta_tick);
        if player.is_changed_after(Tick::new(latest)) || meta.is_changed_after(timer.last_changed())
        {
            let duration = player.interval.unwrap_or_else(|| {
                Duration::from_millis(meta.0.map(|meta| meta.interval).unwrap_or_default() as u64)
            });
            timer.timer = Timer::new(duration, TimerMode::Repeating);
            timer.initialized = true;
        }
    });
}

fn sprite_sheet_player(
    mut q_player: Query<(
        &SpriteSheetIndex,
        &SpriteSheetPlayer,
        &mut SpriteSheetPlayerTimer,
        &SpriteSheetMeta,
    )>,
    time: Res<Time>,
) {
    q_player
        .par_iter_mut()
        .for_each(|(index, player, mut timer, meta)| {
            if timer.initialized
                && timer.tick(time.delta()).just_finished()
                && let Some(meta) = meta.as_ref()
                && meta.count > 0
            {
                let current_index = index.get();
                let next_index = current_index + 1;
                let next_index = next_index % meta.count as usize;
                if player.repeat || next_index != 0 {
                    index.set(next_index);
                }
            }
        });
}

/// Queues sprite sheet changes in a stack-like structure, where only the top priority item takes effect.
#[derive(Component, Debug, Default)]
#[require(SpriteSheet)]
pub struct SpriteSheetQueue {
    set: BTreeMultiSet<QueueItem>,
}
impl SpriteSheetQueue {
    pub fn new_with(name: impl Into<String>) -> Self {
        let mut queue = Self::default();
        queue.push(0, name);
        queue
    }
    pub fn push(&mut self, priority: u32, name: impl Into<String>) {
        self.set.insert(QueueItem {
            priority,
            name: name.into(),
        });
    }
    pub fn pop(&mut self, priority: u32, name: impl Into<String>) -> bool {
        let item = QueueItem {
            priority,
            name: name.into(),
        };
        self.set.remove(&item)
    }
}
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
struct QueueItem {
    priority: u32,
    name: String,
}
/// Convenient event call to update a specific sprite sheet
#[derive(Debug, EntityEvent)]
pub struct SpriteSheetQueueUpdate {
    pub entity: Entity,
    pub priority: u32,
    pub name: String,
    /// Push if false, pop if true
    pub pop: bool,
}
impl SpriteSheetQueueUpdate {
    pub fn new(entity: Entity, priority: u32, name: impl Into<String>) -> Self {
        Self {
            entity,
            priority,
            name: name.into(),
            pop: false,
        }
    }
    pub fn pop(mut self) -> Self {
        self.pop = true;
        self
    }
}

fn update_sprite_sheet_queue(
    event: On<SpriteSheetQueueUpdate>,
    mut q_queue: Query<&mut SpriteSheetQueue>,
) {
    let Ok(mut queue) = q_queue.get_mut(event.entity) else {
        return;
    };
    if event.pop {
        queue.pop(event.priority, event.name.clone());
    } else {
        queue.push(event.priority, event.name.clone());
    }
}

fn queue_sprite_sheet(
    mut q_sprite_sheet: Query<(&mut SpriteSheet, &SpriteSheetIndex, &SpriteSheetQueue)>,
) {
    q_sprite_sheet
        .par_iter_mut()
        .for_each(|(mut sprite_sheet, index, queue)| {
            if let Some(last) = queue.set.last()
                && sprite_sheet.name != last.name
            {
                sprite_sheet.name = last.name.clone();
                // TODO: Is it necessary to allow custom starting positions?
                index.set(0);
            }
        });
}
