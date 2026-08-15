use super::*;

pub(super) struct ProjectilesPlugin;

impl Plugin for ProjectilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            bread_slice_fade.run_if(in_state(BREAD_STATE.clone())),
        );
        app.world_mut()
            .register_component_hooks::<BreadSlice>()
            .on_add(|mut world, HookContext { entity, .. }| {
                let timer = world.get::<BreadSlice>(entity);
                let immediate = timer
                    .is_some_and(|timer| timer.fade.duration() < Duration::from_secs_f32(1e-3));
                if !immediate {
                    world.commands().entity(entity).insert(ColliderDisabled);
                }
            });
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
    pub accelerate: Timer,
    pub target_velocity: Vec2,
    pub track_player: bool,
    /// Whether to play sound when fade-in is complete.
    pub play_sound: bool,
}
impl Default for BreadSlice {
    fn default() -> Self {
        let mut default_timer = Timer::from_seconds(1.0, TimerMode::Once);
        default_timer.almost_finish();
        Self {
            fade: default_timer.clone(),
            accelerate: default_timer,
            target_velocity: vec2(400.0, 0.0),
            track_player: false,
            play_sound: false,
        }
    }
}
impl BreadSlice {
    pub fn with_angle(mut self, angle: f32) -> Self {
        self.target_velocity = self.target_velocity.rotate(Vec2::from_angle(angle));
        self
    }
    pub fn with_rotate(mut self, rotate: Vec2) -> Self {
        self.target_velocity = self.target_velocity.rotate(rotate);
        self
    }
    pub fn with_velocity_multiply(mut self, multiplier: f32) -> Self {
        self.target_velocity *= multiplier;
        self
    }
    pub fn with_fade(mut self, secs: f32) -> Self {
        self.fade = Timer::from_seconds(secs, TimerMode::Once);
        self
    }
    pub fn with_accelerate(mut self, secs: f32) -> Self {
        self.accelerate = Timer::from_seconds(secs, TimerMode::Once);
        self
    }
    pub fn with_play_sound(mut self) -> Self {
        self.play_sound = true;
        self
    }
    pub fn with_track_player(mut self) -> Self {
        self.track_player = true;
        self
    }
}

fn bread_slice_fade(
    mut q_slice: Query<(
        Entity,
        &GlobalTransform,
        &mut BreadSlice,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &mut Sprite,
    )>,
    time: Res<Time>,
    commands: ParallelCommands,
    player: Option<Res<player::PrimaryPlayer>>,
    q_transform: Query<&GlobalTransform>,
) {
    let Some(player) = player else {
        return;
    };
    let Ok(global_transform) = q_transform.get(player.0) else {
        return;
    };
    let player_position = global_transform.translation().xy();
    q_slice.par_iter_mut().for_each(
        |(
            entity,
            global_transform,
            mut slice,
            mut linear_velocity,
            mut angular_velocity,
            mut sprite,
        )| {
            if !slice.fade.is_finished() {
                if slice.fade.tick(time.delta()).just_finished() {
                    if slice.track_player {
                        let position = global_transform.translation().xy();
                        slice.target_velocity = (player_position - position)
                            .normalize_or(vec2(1.0, 0.0))
                            * slice.target_velocity.length();
                    }
                    if slice.play_sound {
                        commands.command_scope(|mut commands| {
                            commands.spawn(Sound::new("Bread_Shoot"));
                        });
                    }
                    commands.command_scope(|mut commands| {
                        commands.entity(entity).remove::<ColliderDisabled>();
                    });
                    sprite.color.set_alpha(1.0);
                } else {
                    sprite.color.set_alpha(slice.fade.fraction());
                }
            } else if !slice.accelerate.is_finished() {
                **linear_velocity = slice.target_velocity * slice.accelerate.fraction();
                angular_velocity.0 =
                    -linear_velocity.x.signum() * linear_velocity.length() / SLICE_HALF_LENGTH;
                slice.accelerate.tick(time.delta());
            }
        },
    );
}
