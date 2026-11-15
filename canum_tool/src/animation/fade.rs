use crate::prelude::*;

pub(super) struct FadePlugin;

impl Plugin for FadePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            sprite_sheet_fade.in_set(super::AnimationSystems::Update),
        );
    }
}

/// Fades the alpha of a sprite sheet over time.
///
/// The alpha starts at `starting_alpha` when this component is added, then
/// interpolates to `ending_alpha` over `time`. When the fade finishes, this
/// component removes itself. If `despawn` is `true`, the
/// entity is despawned instead.
#[derive(Component, Debug, Clone)]
#[require(Sprite, SpriteSheetFadeClock)]
pub struct SpriteFade {
    /// How long the fade takes.
    pub time: Duration,
    /// The alpha at the beginning of the fade.
    pub starting_alpha: f32,
    /// The alpha at the end of the fade.
    pub ending_alpha: f32,
    /// Despawns the entity after a fade out finishes. This has no effect on fade in.
    pub despawn: bool,
}
impl SpriteFade {
    /// Creates a fade from transparent to opaque.
    pub fn fade_in() -> Self {
        Self {
            time: Duration::from_secs_f32(0.25),
            starting_alpha: 0.0,
            ending_alpha: 1.0,
            despawn: false,
        }
    }
    /// Creates a fade from opaque to transparent.
    pub fn fade_out() -> Self {
        Self {
            time: Duration::from_secs_f32(0.25),
            starting_alpha: 1.0,
            ending_alpha: 0.0,
            despawn: true,
        }
    }

    pub fn with_time(mut self, time: Duration) -> Self {
        self.time = time;
        self
    }
    pub fn with_alpha(mut self, starting_alpha: f32, ending_alpha: f32) -> Self {
        self.starting_alpha = starting_alpha;
        self.ending_alpha = ending_alpha;
        self
    }
    pub fn with_starting_alpha(mut self, starting_alpha: f32) -> Self {
        self.starting_alpha = starting_alpha;
        self
    }
    pub fn with_ending_alpha(mut self, ending_alpha: f32) -> Self {
        self.ending_alpha = ending_alpha;
        self
    }
    pub fn with_despawn(mut self) -> Self {
        self.despawn = true;
        self
    }
}
impl Default for SpriteFade {
    fn default() -> Self {
        Self::fade_out()
    }
}

#[derive(Component, Debug, Deref, DerefMut)]
struct SpriteSheetFadeClock {
    #[deref]
    timer: Timer,
}
impl Default for SpriteSheetFadeClock {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_secs_f32(0.5), TimerMode::Once),
        }
    }
}

fn sprite_sheet_fade(
    mut q_fade: Query<(
        Entity,
        Ref<SpriteFade>,
        &mut Sprite,
        &mut SpriteSheetFadeClock,
    )>,
    time: Res<Time>,
    commands: ParallelCommands,
) {
    q_fade
        .par_iter_mut()
        .for_each(|(entity, fade, mut sprite, mut clock)| {
            if fade.is_changed() {
                clock.timer = Timer::new(fade.time, TimerMode::Once);
                sprite.color.set_alpha(fade.starting_alpha.clamp(0.0, 1.0));
                return;
            }

            clock.timer.tick(time.delta());
            let fraction = clock.timer.fraction();
            let alpha = fade.starting_alpha + (fade.ending_alpha - fade.starting_alpha) * fraction;
            sprite.color.set_alpha(alpha.clamp(0.0, 1.0));

            if clock.timer.just_finished() {
                commands.command_scope(|mut commands| {
                    if fade.despawn {
                        commands.entity(entity).try_despawn();
                    } else {
                        commands
                            .entity(entity)
                            .remove::<SpriteFade>()
                            .remove::<SpriteSheetFadeClock>();
                    }
                });
            }
        });
}
