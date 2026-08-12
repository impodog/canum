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
        app.add_systems(OnEnter(WINDY_STATE.clone()), |mut commands: Commands| {
            commands.insert_resource(WindVelocity::default());
        });
        app.add_systems(OnExit(WINDY_STATE.clone()), |mut commands: Commands| {
            commands.remove_resource::<WindVelocity>();
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
            target_velocity: Vec2::new(250.0, 0.0),
            friction: 0.25,
        }
    }
}

/// Auto spawn as children of `CanBeBlown`.
#[derive(Component, Default)]
#[require(movements::PartialVelocity::unlinked())]
pub struct WindForcedVelocity;

#[derive(Component, Default)]
pub struct CanBeBlown;

fn update_wind_forced_velocity(
    wind_velocity: Res<WindVelocity>,
    mut q_velocity: Query<&mut movements::PartialVelocity, With<WindForcedVelocity>>,
    time: Res<Time>,
) {
    q_velocity.par_iter_mut().for_each(|mut partial_velocity| {
        let diff = wind_velocity.target_velocity - **partial_velocity;
        **partial_velocity += diff * wind_velocity.friction * time.delta_secs();
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
                .insert((CanBeBlown, movements::ForcedVelocity::default()));
        });
    });
}
