use super::*;

pub(super) struct WindPlugin;

impl Plugin for WindPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<CanBeBlown>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .spawn((ChildOf(entity), WindForcedVelocity));
            });
        app.add_systems(
            FixedPreUpdate,
            (update_wind_forced_velocity, add_for_projectiles)
                .run_if(in_state(WINDY_STATE.clone())),
        );
        app.add_systems(
            FixedUpdate,
            spawn_particles.run_if(in_state(WINDY_STATE.clone())),
        );
        app.add_systems(OnEnter(WINDY_STATE.clone()), |mut commands: Commands| {
            commands.insert_resource(WindVelocity::default());
            commands.insert_resource(WindyParticleTimer::default());
        });
        app.add_systems(OnExit(WINDY_STATE.clone()), |mut commands: Commands| {
            commands.remove_resource::<WindVelocity>();
            commands.remove_resource::<WindyParticleTimer>();
        });
    }
}

/// Global resources that controls the wind blowing strength.
#[derive(Resource, Debug)]
pub struct WindVelocity {
    pub target_velocity: Vec2,
    pub friction: f32,
}
impl Default for WindVelocity {
    fn default() -> Self {
        Self {
            target_velocity: Vec2::new(225.0, 0.0),
            friction: 0.25,
        }
    }
}

/// Auto spawn as children of `CanBeBlown`.
#[derive(Component, Default)]
#[require(movements::PartialVelocity::unlinked(), movements::SpeedShrinkExclude)]
pub struct WindForcedVelocity;

/// Defines the blow strength multiplier.
#[derive(Component)]
pub struct CanBeBlown(pub f32);
impl Default for CanBeBlown {
    fn default() -> Self {
        Self(1.0)
    }
}

fn update_wind_forced_velocity(
    wind_velocity: Res<WindVelocity>,
    mut q_velocity: Query<(&mut movements::PartialVelocity, &ChildOf), With<WindForcedVelocity>>,
    q_can_be_blown: Query<&CanBeBlown>,
    time: Res<Time>,
) {
    q_velocity
        .par_iter_mut()
        .for_each(|(mut partial_velocity, parent)| {
            let Ok(can_be_blown) = q_can_be_blown.get(parent.0) else {
                return;
            };
            let target_velocity = wind_velocity.target_velocity * can_be_blown.0;
            if partial_velocity.x.signum() != target_velocity.x.signum() {
                partial_velocity.x = 0.0;
            }
            let diff = target_velocity - **partial_velocity;
            **partial_velocity +=
                diff * wind_velocity.friction * can_be_blown.0.abs() * time.delta_secs();
        });
}

fn add_for_projectiles(
    q_projectile: Query<Entity, Added<projectile::Projectile>>,
    commands: ParallelCommands,
) {
    q_projectile.par_iter().for_each(|entity| {
        commands.command_scope(|mut commands| {
            commands
                .entity(entity)
                .insert((CanBeBlown(1.5), movements::ForcedVelocity::default()));
        });
    });
}

#[derive(Component, Default)]
#[require(
    SessionOnly,
    CanBeBlown(3.0),
    RigidBody::Kinematic,
    movements::ForcedVelocity,
    projectile::RemoveOutOfBounds,
    StageDelete
)]
struct WindyParticle;

#[derive(Resource, Deref, DerefMut)]
struct WindyParticleTimer(Timer);
impl Default for WindyParticleTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}

fn spawn_particles(
    mut commands: Commands,
    wind: Option<Res<WindVelocity>>,
    mut timer: ResMut<WindyParticleTimer>,
    time: Res<Time>,
    q_player_can_be_blown: Query<(), (With<player::Player>, With<CanBeBlown>)>,
) {
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    if q_player_can_be_blown.iter().next().is_none() {
        return;
    }
    let Some(wind) = wind else {
        return;
    };
    if wind.target_velocity.x.abs() < 50.0 {
        return;
    }
    let sgn = wind.target_velocity.x.signum();
    let x = CONFIG.display.half_virtual_size.0 + 16.0;
    commands.spawn((
        WindyParticle,
        Animation::new("Windy_Particle", vec2(16.0, 16.0)).with_color(Color::WHITE.with_alpha(0.3)),
        Transform::from_translation(vec3(
            x * -sgn,
            rand::random_range(
                -CONFIG.display.half_virtual_size.1..CONFIG.display.half_virtual_size.1,
            ),
            0.0,
        )),
        Sprite {
            flip_x: sgn.is_sign_positive(),
            flip_y: rand::random_bool(0.5),
            ..default()
        },
    ));
}
