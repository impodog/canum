use super::*;
use enemy::behavior::*;

pub(super) struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<ShootAndRotate>()
            .on_insert(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(shoot_and_rotate_start)
                    .observe(shoot_and_rotate_begin);
            });
        app.add_systems(
            FixedUpdate,
            shoot_and_rotate_track_player.run_if(in_state(LASER_STATE.clone())),
        );
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct LaserBehaviors;

#[derive(Component, Default)]
#[require(Behavior::new("Laser_ShootAndRotate", 0.8, ["Animation", "Rotation", "ShootAndRotate"]))]
pub struct ShootAndRotate {
    target: Option<Entity>,
    timer: Timer,
}
const LASER_SIZE: Vec2 = vec2(32.0, 10.0);

fn shoot_and_rotate_start(
    event: On<BehaveStart>,
    mut q_timer: Query<&mut ShootAndRotate>,
    mut q_animation: Query<&mut Animation>,
) {
    let Ok(mut timer) = q_timer.get_mut(event.entity) else {
        return;
    };
    timer.target = Some(event.target);
    let Ok(mut animation) = q_animation.get_mut(event.target) else {
        return;
    };
    animation.replace(
        "Laser_Prepare",
        true,
        Some(AnimationInform {
            entity: event.entity,
            index: vec![usize::MAX],
        }),
    );
}

fn shoot_and_rotate_begin(
    event: On<AnimationComplete>,
    mut q_timer: Query<&mut ShootAndRotate>,
    mut commands: Commands,
) {
    let Ok(mut timer) = q_timer.get_mut(event.entity) else {
        return;
    };
    timer.timer = Timer::from_seconds(5.0, TimerMode::Once);
    commands
        .spawn((
            ChildOf(event.entity),
            Transform::from_translation(vec3(20.0, 0.0, 0.0)),
        ))
        .observe(laser_shoot_spawn)
        .insert(canum_fx::weapon::LaserLike {
            middle: Animation::new("Laser_AttackMiddle", LASER_SIZE),
            terminal: Animation::new("Laser_AttackTerminal", LASER_SIZE),
            collider: Collider::rectangle(LASER_SIZE.x, LASER_SIZE.y - 1.0),
            length: LASER_SIZE.x,
        });
}

fn laser_shoot_spawn(event: On<canum_fx::weapon::LaserSpawn>, mut commands: Commands) {
    commands.entity(event.spawn).insert((
        health::ContactDamage {
            value: 50,
            projectile: false,
            order: consts::order::PLAYER_PROJ,
        },
        health::Friendly::UNFRIENDLY,
    ));
}

fn shoot_and_rotate_track_player(
    mut q_timer: Query<(
        Entity,
        &mut ShootAndRotate,
        &mut Transform,
        &GlobalTransform,
    )>,
    mut q_animation: Query<&mut Animation>,
    q_global_transform: Query<&GlobalTransform>,
    player: Option<Res<player::PrimaryPlayer>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let Some(player) = player else {
        return;
    };
    let Ok(player_transform) = q_global_transform.get(player.0) else {
        return;
    };
    let player_position = player_transform.translation().xy();
    for (entity, mut timer, mut transform, global_transform) in q_timer.iter_mut() {
        if timer.timer.tick(time.delta()).just_finished() {
            commands.trigger(BehaveEnd {
                entity,
                cooldown: Duration::from_secs_f32(rand_normal(1.0, 0.2)),
                occupies: occupies![("ShootAndRotate", 2.0)],
            });
            let Some(target) = timer.target else {
                continue;
            };
            let Ok(mut animation) = q_animation.get_mut(target) else {
                continue;
            };
            animation.replace("Laser_Static", false, None);
            commands.entity(entity).despawn_children();
            continue;
        }
        let position = global_transform.translation().xy();
        let diff = player_position - position;
        let angle = diff.to_angle();
        let angle_diff =
            normalize_angle_signed(angle - global_transform.rotation().to_euler(EulerRot::XYZ).2);
        transform.rotate_z(angle_diff * time.delta_secs() * 2.5);
    }
}
