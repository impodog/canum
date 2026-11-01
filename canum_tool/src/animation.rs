use bevy::ecs::change_detection::Tick;

use crate::prelude::*;
use canum_save::util::BTreeMultiSet;

pub(super) struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (init_sprite_sheet_player,).in_set(AnimationSystems::Init),
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
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationSystems {
    Init,
    Update,
    Final,
}

/// Auto advances the sprite sheet index. You can override the interval if needed.
#[derive(Component, Default, Debug)]
#[require(SpriteSheetPlayerTimer)]
pub struct SpriteSheetPlayer {
    pub interval: Option<Duration>,
    pub repeat: bool,
}
#[derive(Component, Default, Deref, DerefMut, Debug)]
struct SpriteSheetPlayerTimer {
    #[deref]
    timer: Timer,
    timestamp: u32,
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
        if player.is_changed_after(Tick::new(latest)) {
            let duration = player.interval.unwrap_or_else(|| {
                Duration::from_millis(meta.0.map(|meta| meta.interval).unwrap_or_default() as u64)
            });
            timer.timer = Timer::new(duration, TimerMode::Repeating);
            timer.timestamp = latest;
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
                if player.repeat || next_index == 0 {
                    index.set(next_index);
                }
            }
        });
}

#[derive(Component, Debug, Default)]
pub struct SpriteSheetQueue {
    set: BTreeMultiSet<QueueItem>,
}
impl SpriteSheetQueue {
    pub fn push(&mut self, priority: u32, name: impl Into<String>) {
        self.set.insert(QueueItem {
            priority,
            name: name.into(),
        });
    }
}
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
struct QueueItem {
    priority: u32,
    name: String,
}
