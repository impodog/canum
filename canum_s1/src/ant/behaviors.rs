use crate::prelude::*;
use enemy::behavior::*;

pub(super) struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(super::ANT_STATE.clone()),
            |mut commands: Commands| {
                commands.insert_resource(AntStage::default());
            },
        );
        app.add_systems(
            OnExit(super::ANT_STATE.clone()),
            |mut commands: Commands| {
                commands.remove_resource::<AntStage>();
            },
        );
        app.world_mut()
            .register_component_hooks::<AntBehaviors>()
            .on_add(|mut world, HookContext { entity, .. }| {
                let mut commands = world.commands();
                commands
                    .spawn((ChildOf(entity), RunToPlayer::default()))
                    .observe(run_to_player_start)
                    .observe(run_to_player_end);
            });
    }
}

/// Stores stage-related values.
#[derive(Resource, Debug)]
pub struct AntStage {
    pub stage: u8,
}

impl Default for AntStage {
    fn default() -> Self {
        Self { stage: 1 }
    }
}

pub(super) fn change_ant_stage(
    event: On<health::Damage>,
    mut commands: Commands,
    mut ant_stage: ResMut<AntStage>,
    q_health: Query<&enemy::health::EnemyHealth>,
    q_behaviors: Query<Entity, With<AntBehaviors>>,
) {
    let Ok(health) = q_health.get(event.entity) else {
        return;
    };
    let new_stage = if health.value >= 2700 {
        1
    } else if health.value >= 1500 {
        2
    } else if health.value >= 600 {
        3
    } else {
        4
    };
    if ant_stage.stage != new_stage {
        ant_stage.stage = new_stage;
    }
    for behaviors in q_behaviors.iter() {
        match new_stage {
            2 => {
                commands
                    .spawn((ChildOf(behaviors), ThrowBlade))
                    .observe(throw_blade_start)
                    .observe(throw_blade_end);
            }
            _ => {}
        }
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct AntBehaviors;

#[derive(Component, Default)]
#[require(Behavior::new("Ant_RunToPlayer", 1.0, ["RunToPlayer", "Displacement", "Animation"]), BaseFartherBetter::new(0.0, 200.0))]
struct RunToPlayer {
    target: Option<Entity>,
}

fn run_to_player_start(
    event: On<BehaveStart>,
    mut commands: Commands,
    mut q_behavior: Query<&mut RunToPlayer>,
    mut q_ant: Query<(&mut Animation, &GlobalTransform)>,
    q_player: Query<(&GlobalTransform, &LinearVelocity)>,
    player: Res<player::RandomPlayer>,
) {
    {
        let Ok(mut behavior) = q_behavior.get_mut(event.entity) else {
            return;
        };
        behavior.target = Some(event.target);
    }
    let Ok((mut animation, ant_transform)) = q_ant.get_mut(event.target) else {
        return;
    };
    let Ok((player_transform, player_velocity)) = q_player.get(player.0) else {
        return;
    };
    let ant_position = ant_transform.translation().xy();
    let player_position = player_transform.translation().xy();
    let displace =
        (player_position - ant_position) * 1.1 + player_velocity.0 * rand_normal(0.2, 0.1);
    let duration = (displace.length() / rand_normal(130.0, 5.0)).min(2.0);
    animation.replace("Ant_Run", false, None);
    commands.spawn((
        ChildOf(event.target),
        enemy::movements::Displacement {
            curve: |x| CubicOutCurve.sample(x).unwrap(),
            displace,
            duration: Duration::from_secs_f32(duration),
            notify: Some(event.entity),
        },
    ));
    commands.spawn((ChildOf(event.entity), Sound::new("Ant_Run")));
}

fn run_to_player_end(
    event: On<enemy::movements::DisplacementComplete>,
    mut commands: Commands,
    mut q_ant: Query<&mut Animation>,
    q_behavior: Query<&RunToPlayer>,
) {
    let Ok(behavior) = q_behavior.get(event.entity) else {
        return;
    };
    let Some(target) = behavior.target else {
        return;
    };
    let Ok(mut animation) = q_ant.get_mut(target) else {
        return;
    };
    animation.replace("Ant_Static", false, None);
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(0.5),
        occupies: occupies![("RunToPlayer", rand_normal(3.0, 0.5))],
    });
    commands.entity(event.entity).despawn_children();
}

#[derive(Component, Default)]
#[require(Behavior::new("Ant_ThrowBlade", 1.0, ["ThrowBlade", "Displacement", "Animation"]), MultiplierBySpeed::new(100.0, 5.0))]
struct ThrowBlade;

fn throw_blade_start(
    event: On<BehaveStart>,
    mut commands: Commands,
    mut q_ant: Query<(&mut Animation, &movements::AutoFlipStatus, &GlobalTransform)>,
) {
    let Ok((mut animation, flip_status, ant_transform)) = q_ant.get_mut(event.target) else {
        return;
    };
    let mut translation = ant_transform.translation();
    translation.z += 0.1;
    translation.x += flip_status.x * 40.0;
    let blade = commands
        .spawn((
            Blade { released: false },
            Transform::from_translation(translation),
        ))
        .id();
    commands.spawn((
        ChildOf(blade),
        enemy::movements::RotationAround {
            around: event.target,
            angular_velocity: std::f32::consts::FRAC_PI_4 * flip_status.x,
        },
    ));
    animation.replace(
        "Ant_Throw",
        true,
        Some(AnimationInform {
            entity: event.entity,
            index: vec![4, 0],
        }),
    );
}

fn throw_blade_end(
    event: On<AnimationComplete>,
    mut commands: Commands,
    mut q_animation: Query<&mut Animation>,
    mut q_blade: Query<(
        Entity,
        &mut movements::ForcedVelocity,
        &mut Blade,
        &GlobalTransform,
    )>,
    q_transform: Query<&GlobalTransform>,
    q_velocity: Query<&LinearVelocity>,
    player: Res<player::RandomPlayer>,
) {
    if event.index == 0 {
        let Ok(mut animation) = q_animation.get_mut(event.source) else {
            return;
        };
        animation.replace("Ant_Static", false, None);
        commands.trigger(BehaveEnd {
            entity: event.entity,
            cooldown: Duration::from_secs_f32(rand_normal(1.0, 0.3)),
            occupies: occupies![("ThrowBlade", rand_normal(2.0, 0.4))],
        });
    } else {
        let Ok(player_transform) = q_transform.get(player.0) else {
            return;
        };
        let Ok(player_velocity) = q_velocity.get(player.0) else {
            return;
        };
        let player_position = player_transform.translation().xy();
        for (entity, mut forced_velocity, mut blade, transform) in q_blade.iter_mut() {
            commands.entity(entity).despawn_children();
            if !blade.released {
                let position = transform.translation().xy();
                let direction =
                    player_position - position + player_velocity.0 * rand_normal(0.4, 0.2);
                forced_velocity.0 = direction.normalize_or_zero() * 350.0;
                blade.released = true;
            }
            commands.spawn((ChildOf(entity), Sound::new("Turf_Shooter_Shoot")));
        }
    }
}

#[derive(Component, Default)]
#[require(
    Animation::new("Ant_Blade", vec2(45.0, 45.0)),
    enemy::attack::EnemyProjectile,
    movements::ForcedVelocity,
    Collider::circle(20.0),
    Mass(5.0)
)]
struct Blade {
    released: bool,
}
