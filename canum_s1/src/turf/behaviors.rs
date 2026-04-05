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
            (turf_spiker_move, turf_shooter_move).run_if(in_state(super::TURF_STATE.clone())),
        );
        app.add_systems(
            FixedPostUpdate,
            (remove_turf_shooter,).run_if(in_state(super::TURF_STATE.clone())),
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

fn update_turf_stage(
    mut commands: Commands,
    mut stage: ResMut<TurfStage>,
    q_turf: Query<&super::TurfBoss>,
    q_behaviors: Query<Entity, With<TurfBehaviors>>,
) {
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
    let Ok(behaviors) = q_behaviors.single() else {
        return;
    };
    if stage.0 != stage_should_be {
        stage.0 = stage_should_be;
        if stage.0 == 1 {
            commands
                .spawn((ChildOf(behaviors), TurfSpawnShooter))
                .observe(spawn_shooter_start);
        }
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
        -CONFIG.display.half_virtual_size.1 + 45.0,
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
            Animation::new("Turf_Spiker1_Run", vec2(64.0, 64.0)),
        ));
    } else {
        commands.spawn((
            TurfSpiker {
                free: false,
                fixed: generate_fixed(0.8),
                tolerance: rand_normal(0.0, 20.0),
            },
            Transform::from_translation(spawn_position),
            Animation::new("Turf_Spiker2_Run", vec2(64.0, 64.0)),
        ));
    }
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: default(),
        occupies: occupies![("SpawnSpiker", rand_normal(0.45, 0.05))],
    });
}

#[derive(Component, Default)]
#[require(
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
                    let speed = rand_normal(300.0, 25.0);
                    // Shoot towards player
                    if (stage.0 >= 2 && rand::random_bool(0.15))
                        || (stage.0 >= 3 && rand::random_bool(0.15))
                    {
                        **forced_velocity =
                            (player_position - position).normalize_or_zero() * speed;
                        transform
                            .rotate_z(forced_velocity.to_angle() - std::f32::consts::FRAC_PI_2);
                    } else {
                        let speed = if player_position.y <= position.y {
                            -speed
                        } else {
                            speed
                        };
                        **forced_velocity = Vec2::new(0.0, speed);
                    }
                    let new_name = animation.name.replace("Run", "Spin");
                    animation.replace(new_name, false, None);
                }
            }
        },
    );
}

#[derive(Component, Debug, Clone)]
#[require(Behavior::new("Turf_SpawnShooter", 1.0, ["SpawnShooter"]))]
pub struct TurfSpawnShooter;

fn spawn_shooter_start(event: On<BehaveStart>, mut commands: Commands) {
    let x = rand::random_range(
        -CONFIG.display.half_virtual_size.0 + 50.0..CONFIG.display.half_virtual_size.0 - 50.0,
    );
    commands
        .spawn((
            TurfShooter,
            Transform::from_translation(Vec3::new(x, -CONFIG.display.half_virtual_size.1, 1.0)),
        ))
        .observe(turf_shooter_shoot);
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: default(),
        occupies: occupies![("SpawnShooter", rand_normal(8.0, 1.0))],
    });
}

#[derive(Component, Default)]
#[require(
    enemy::attack::Minion,
    Animation::new("Turf_Shooter_Static", Vec2::new(64.0, 64.0)),
    movements::ForcedVelocity(Vec2::new(0.0, rand_normal(300.0, 10.0))),
    Collider::triangle(vec2(-5.0, -10.0), vec2(5.0, -10.0), vec2(0.0, 10.0)),
    Mass(10.0)
)]
pub struct TurfShooter;

#[derive(Component, Default)]
#[require(
    Animation::new("Turf_Seed", Vec2::new(20.0, 10.0)),
    enemy::attack::EnemyProjectile,
    Collider::rectangle(15.0, 7.5),
    Mass(3.0)
)]
pub struct TurfShooterSeed;

fn turf_shooter_move(
    mut q_shooter: Query<
        (Entity, &mut movements::ForcedVelocity, &mut Animation),
        With<TurfShooter>,
    >,
) {
    const SPEED_DECREASE: f32 = 2.5;
    q_shooter
        .par_iter_mut()
        .for_each(|(entity, mut forced_velocity, mut animation)| {
            forced_velocity.y -= SPEED_DECREASE;
            if forced_velocity.y.abs() <= SPEED_DECREASE {
                animation.replace(
                    "Turf_Shooter_Shoot",
                    true,
                    Some(AnimationInform {
                        entity,
                        index: vec![0, 3],
                    }),
                );
            }
        });
}

fn turf_shooter_shoot(
    event: On<AnimationComplete>,
    mut commands: Commands,
    q_transform: Query<&GlobalTransform>,
    player: Res<player::RandomPlayer>,
    mut q_animation: Query<&mut Animation>,
) {
    if event.index == 0 {
        let Ok(mut animation) = q_animation.get_mut(event.entity) else {
            return;
        };
        animation.replace("Turf_Shooter_Static", false, None);
    } else {
        let Ok(player_transform) = q_transform.get(player.0) else {
            return;
        };
        let player_position = player_transform.translation().xy();
        let Ok(shooter_transform) = q_transform.get(event.entity) else {
            return;
        };
        let shooter_position = shooter_transform.translation().xy();
        let direction = (player_position - shooter_position).to_angle() + rand_normal(0.0, 0.1);
        let offset_angle = rand_normal(0.5, 0.1);
        let transform = Transform::from_translation(shooter_transform.translation());
        for offset in [-offset_angle, offset_angle, 0.0] {
            let velocity = Vec2::from_angle(offset + direction) * 325.0;
            let mut transform = transform;
            transform.rotate_z(offset + direction);
            commands.spawn((TurfShooterSeed, transform, LinearVelocity(velocity)));
        }
        commands.spawn((Sound::new("Turf_Shooter_Shoot"), ChildOf(event.entity)));
    }
}

fn remove_turf_shooter(
    mut commands: Commands,
    q_shooter: Query<(Entity, &GlobalTransform), With<TurfShooter>>,
) {
    for (entity, global_transform) in q_shooter.iter() {
        let translation = global_transform.translation();
        if translation.y < -CONFIG.display.half_virtual_size.1 {
            commands.entity(entity).despawn();
        }
    }
}
