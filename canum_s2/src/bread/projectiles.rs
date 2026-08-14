use super::*;

pub(super) struct ProjectilesPlugin;

impl Plugin for ProjectilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            bread_slice_fade.run_if(in_state(BREAD_STATE.clone())),
        );
    }
}

const SLICE_LENGTH: f32 = 20.0;
const SLICE_HALF_LENGTH: f32 = SLICE_LENGTH * 0.5;

/// The main projectile the bread spawns.
#[derive(Component)]
#[require(
    enemy::attack::EnemyProjectile,
    Animation::new("Bread_Slice", vec2(SLICE_LENGTH, SLICE_LENGTH)),
    Collider::circle(SLICE_HALF_LENGTH * 0.9)
)]
pub struct BreadSlice {
    pub fade: Timer,
    pub target_velocity: Vec2,
}
impl Default for BreadSlice {
    fn default() -> Self {
        let mut fade = Timer::from_seconds(1.0, TimerMode::Once);
        fade.almost_finish();
        Self {
            fade,
            target_velocity: vec2(350.0, 0.0),
        }
    }
}
impl BreadSlice {
    pub fn with_angle(mut self, angle: f32) -> Self {
        self.target_velocity = self.target_velocity.rotate(Vec2::from_angle(angle));
        self
    }
    pub fn with_fade(mut self, secs: f32) -> Self {
        self.fade = Timer::from_seconds(secs, TimerMode::Once);
        self
    }
}

fn bread_slice_fade(
    mut q_slice: Query<(
        &mut BreadSlice,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &mut Sprite,
    )>,
    time: Res<Time>,
) {
    q_slice.par_iter_mut().for_each(
        |(mut slice, mut linear_velocity, mut angular_velocity, mut sprite)| {
            if slice.fade.is_finished() {
                return;
            }
            if slice.fade.tick(time.delta()).just_finished() {
                **linear_velocity = slice.target_velocity;
                **angular_velocity = -linear_velocity.x.signum() * slice.target_velocity.length()
                    / SLICE_HALF_LENGTH;
                sprite.color.set_alpha(1.0);
            } else {
                sprite.color.set_alpha(slice.fade.fraction());
            }
        },
    );
}
