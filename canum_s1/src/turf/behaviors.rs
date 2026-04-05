use crate::prelude::*;
use enemy::behavior::*;

pub(super) struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TurfStage>();
        app.world_mut()
            .register_component_hooks::<TurfBehaviors>()
            .on_add(|mut world, HookContext { entity, .. }| {
                let mut commands = world.commands();
                commands
                    .spawn((ChildOf(entity), TurfSpawnSpiker))
                    .observe(spawn_spiker_start);
            });
        app.add_observer(init_turf_stage);
        app.add_systems(
            FixedPreUpdate,
            update_turf_stage.run_if(in_state(super::TURF_STATE.clone())),
        );
        app.add_systems(
            FixedUpdate,
            (turf_spiker_move,).run_if(in_state(super::TURF_STATE.clone())),
        );
    }
}

#[derive(Component, Debug, Clone)]
#[require(BehaviorManager::new())]
pub struct TurfBehaviors;

#[derive(Resource, Debug, Default)]
pub struct TurfStage(pub u8);

fn init_turf_stage(_event: On<super::background::TurfSetupTimerComplete>, mut commands: Commands) {
    commands.insert_resource(TurfStage::default());
}

fn update_turf_stage(mut stage: ResMut<TurfStage>, q_turf: Query<&super::TurfBoss>) {
    let Ok(turf) = q_turf.single() else {
        return;
    };
    let ratio = turf.timer.elapsed_secs() / turf.timer.duration().as_secs_f32();
    let stage_should_be: u8 = if ratio < 0.22 {
        0
    } else if ratio < 0.5 {
        1
    } else if ratio < 0.75 {
        2
    } else {
        3
    };
    if stage.0 != stage_should_be {
        stage.0 = stage_should_be;
    }
}

#[derive(Component, Debug, Clone)]
#[require(Behavior::new("Turf_SpawnSpiker", 2.0, ["SpawnSpiker"]))]
pub struct TurfSpawnSpiker;

fn spawn_spiker_start(event: On<BehaveStart>, mut commands: Commands) {
    fn generate_fixed(prob: f64) -> Option<f32> {
        if rand::random_bool(prob) {
            Some(rand::random_range(
                50.0 - CONFIG.display.half_virtual_size.0
                    ..CONFIG.display.half_virtual_size.0 - 50.0,
            ))
        } else {
            None
        }
    }
    let spawn_position = Vec3::new(
        -CONFIG.display.half_virtual_size.0,
        -CONFIG.display.half_virtual_size.1 + 32.0,
        -0.1,
    );
    if rand::random_bool(0.5) {
        commands.spawn((
            TurfSpiker {
                free: false,
                fixed: generate_fixed(0.2),
                tolerance: rand_normal(0.0, 10.0),
            },
            Transform::from_translation(spawn_position),
            Animation::new("Turf_Spiker1_Run", vec2(50.0, 50.0)),
        ));
    } else {
        commands.spawn((
            TurfSpiker {
                free: false,
                fixed: generate_fixed(0.8),
                tolerance: rand_normal(0.0, 20.0),
            },
            Transform::from_translation(spawn_position),
            Animation::new("Turf_Spiker2_Run", vec2(50.0, 50.0)),
        ));
    }
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: default(),
        occupies: occupies![("SpawnSpiker", rand_normal(0.35, 0.05))],
    });
}

#[derive(Component, Default)]
#[require(
    SessionOnly,
    Animation,
    enemy::attack::EnemyProjectile,
    movements::ForcedVelocity(Vec2::new(250.0, 0.0)),
    Collider::triangle(vec2(-5.0, -10.0), vec2(5.0, -10.0), vec2(0.0, 10.0)),
    Mass(5.0)
)]
struct TurfSpiker {
    /// If the spiker has already entered free moving stage.
    free: bool,
    /// If the spiker flys at a fixed spot.
    fixed: Option<f32>,
    /// How far the horizontal distance with player will the spiker shoot.
    tolerance: f32,
}

fn turf_spiker_move(
    mut q_spiker: Query<(
        &mut TurfSpiker,
        &mut movements::ForcedVelocity,
        &mut Animation,
        &mut Transform,
        &GlobalTransform,
    )>,
    q_transform: Query<&GlobalTransform>,
    player: Res<player::RandomPlayer>,
    stage: Res<TurfStage>,
) {
    let Ok(player_transform) = q_transform.get(player.0) else {
        return;
    };
    let player_position = player_transform.translation().xy();
    q_spiker.par_iter_mut().for_each(
        |(mut spiker, mut forced_velocity, mut animation, mut transform, global_transform)| {
            let position = global_transform.translation().xy();
            if !spiker.free {
                let release = if let Some(fixed) = spiker.fixed {
                    position.x >= fixed
                } else {
                    position.x - player_position.x >= spiker.tolerance
                };
                if release {
                    spiker.free = true;
                    let speed = rand_normal(325.0, 25.0);
                    // Shoot towards player
                    if stage.0 >= 2 && rand::random_bool(0.2) {
                        **forced_velocity =
                            (player_position - position).normalize_or_zero() * speed;
                        transform
                            .rotate_z(forced_velocity.to_angle() - std::f32::consts::FRAC_PI_2);
                    } else {
                        **forced_velocity = Vec2::new(0.0, speed);
                    }
                    let new_name = animation.name.replace("Run", "Spin");
                    animation.replace(new_name, false, None);
                }
            }
        },
    );
}
