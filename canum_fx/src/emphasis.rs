use crate::prelude::*;

pub(super) struct EmphasisPlugin;

impl Plugin for EmphasisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, expand_and_fade_out);
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
