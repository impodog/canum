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
    mut ant_stage: ResMut<AntStage>,
    q_health: Query<&enemy::health::EnemyHealth>,
) {
    let Ok(health) = q_health.get(event.entity) else {
        return;
    };
    let new_stage = if health.value <= 1000 {
        3
    } else if health.value <= 1500 {
        2
    } else {
        1
    };
    if ant_stage.stage != new_stage {
        ant_stage.stage = new_stage;
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct AntBehaviors;

#[derive(Component, Default)]
#[require(Behavior::new("Ant_RunToPlayer", 1.5, ["RunToPlayer", "Displacement", "Animation"]))]
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
    commands
        .entity(event.target)
        .insert(enemy::movements::Displacement {
            curve: |x| CubicOutCurve.sample(x).unwrap(),
            displace,
            duration: Duration::from_secs_f32(duration),
            notify: Some(event.entity),
        });
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
        cooldown: Duration::from_secs_f32(0.2),
        occupies: occupies![("Displacement", rand_normal(2.0, 0.3))],
    });
    commands.entity(event.entity).despawn_children();
}
