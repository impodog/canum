use crate::prelude::*;
use canum_play::enemy::behavior::*;

pub(super) struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<AppleBehaviors>()
            .on_add(|mut world, HookContext { entity, .. }| {
                let mut commands = world.commands();
                commands
                    .spawn((ChildOf(entity), MoveCloser))
                    .observe(move_closer_start)
                    .observe(move_closer_end);
                commands
                    .spawn((ChildOf(entity), ThrowSlice))
                    .observe(throw_slice_start)
                    .observe(throw_slice_respond);
            });
        app.add_systems(
            FixedPostUpdate,
            (apple_slice_revolve, apple_skin_rotate).run_if(in_state(super::APPLE_STATE.clone())),
        );
    }
}
#[derive(Component, Debug, Clone)]
#[require(BehaviorManager::new())]
pub struct AppleBehaviors;

#[derive(Component, Debug, Clone, Default)]
#[require(Behavior::new("Apple_MoveCloser", 0.5, ["Displacement"]), BaseFartherBetter::new(150.0, 100.0))]
struct MoveCloser;
fn move_closer_start(
    event: On<BehaveStart>,
    mut commands: Commands,
    player: Res<player::RandomPlayer>,
    q_transform: Query<&GlobalTransform>,
    q_linear_velocity: Query<&LinearVelocity>,
) -> Result<()> {
    let target_transform = q_transform.get(event.target)?;
    let player_transform = q_transform.get(player.0)?;
    let player_velocity = q_linear_velocity.get(player.0)?;
    let displace = player_transform.translation().xy() - target_transform.translation().xy()
        + player_velocity.0 * 0.5;
    let duration = (displace.length() / 100.0).min(2.0);
    commands
        .entity(event.target)
        .insert(enemy::movements::Displacement {
            displace: displace * 0.9,
            duration: Duration::from_secs_f32(duration),
            notify: Some(event.entity),
        });
    Ok(())
}
fn move_closer_end(event: On<enemy::movements::DisplacementComplete>, mut commands: Commands) {
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(1.0),
        occupies: occupies![("Displacement", 1.0)],
    });
}

#[derive(Component, Default)]
#[require(
    Behavior::new("Apple_ThrowSlice", 2.0, ["Animation", "Projectile"]),
    BaseByDistance::new(300.0, 100.0)
)]
pub struct ThrowSlice;
fn throw_slice_start(event: On<BehaveStart>, mut q_animation: Query<&mut Animation>) -> Result<()> {
    let mut animation = q_animation.get_mut(event.target)?;
    animation.replace(
        "Apple_Throw",
        true,
        Some(AnimationInform {
            entity: event.entity,
            index: vec![4, 0],
        }),
    );
    Ok(())
}
fn throw_slice_respond(
    event: On<AnimationComplete>,
    mut commands: Commands,
    q_transform: Query<&GlobalTransform>,
    mut q_animation: Query<&mut Animation>,
    player: Res<player::RandomPlayer>,
) -> Result<()> {
    if event.index == 0 {
        let mut animation = q_animation.get_mut(event.source)?;
        animation.replace("Apple_Static", false, None);
        commands.trigger(BehaveEnd {
            entity: event.entity,
            cooldown: Duration::from_secs_f32(rand_normal(1.5, 0.15)),
            occupies: occupies![],
        });
    } else {
        let target_transform = q_transform.get(event.source)?;
        let player_transform = q_transform.get(player.0)?;
        let distance = player_transform.translation().xy() - target_transform.translation().xy();
        let mut direction = distance.to_angle();
        direction += rand_normal(0.0, 0.1);
        if rand::random_bool(0.99) {
            direction += std::f32::consts::PI;
        }
        let direction = Vec2::from_angle(direction);
        let mut transform = Transform::from_translation(target_transform.translation());
        transform.translation.z += 0.1;
        commands.spawn((
            AppleSlice { direction },
            transform,
            LinearVelocity(direction * 250.0),
        ));
        commands.spawn((ChildOf(event.entity), Sound::new("Apple_Swoosh")));
    }
    Ok(())
}

/// Marks the apple projectile -- slice.
#[derive(Component, Default)]
#[require(
    Animation::new("Apple_Slice", Vec2::new(50.0, 50.0)),
    enemy::attack::EnemyProjectile,
    movements::ForcedVelocity,
    Collider::ellipse(25.0, 20.0),
    Mass(10.0)
)]
struct AppleSlice {
    pub direction: Vec2,
}

fn apple_slice_revolve(mut q_slice: Query<(&AppleSlice, &mut movements::ForcedVelocity)>) {
    q_slice
        .par_iter_mut()
        .for_each(|(apple_slice, mut forced_velocity)| {
            **forced_velocity -= apple_slice.direction * 10.0;
        });
}

#[derive(Component)]
#[require(
    Behavior::new("Apple_PeelSkin", 0.8, ["Animation", "Projectile", "Displacement", "PeelSkin"]),
    BaseByDistance::new(150.0, 80.0)
)]
pub struct PeelSkin {
    pub count: usize,
    pub start_speed: f32,
}
impl Default for PeelSkin {
    fn default() -> Self {
        Self {
            count: 6,
            start_speed: 450.0,
        }
    }
}

fn peel_skin_start(event: On<BehaveStart>, mut q_animation: Query<&mut Animation>) {
    let Ok(mut animation) = q_animation.get_mut(event.target) else {
        return;
    };
    animation.replace(
        "Apple_Peel",
        true,
        Some(AnimationInform {
            entity: event.entity,
            index: vec![0, 3],
        }),
    )
}
fn peel_skin_respond(
    event: On<AnimationComplete>,
    mut commands: Commands,
    mut q_animation: Query<&mut Animation>,
    q_peel_skin: Query<(&GlobalTransform, &PeelSkin)>,
) {
    if event.index == 0 {
        let Ok(mut animation) = q_animation.get_mut(event.source) else {
            return;
        };
        animation.replace("Apple_Static", false, None);
        commands.trigger(BehaveEnd {
            entity: event.entity,
            // There is a long animation after this attack, so no need for long cooldown.
            cooldown: Duration::from_secs_f32(rand_normal(1.2, 0.1)),
            occupies: occupies![("PeelSkin", 2.0)],
        });
    } else {
        let Ok((transform, peel_skin)) = q_peel_skin.get(event.entity) else {
            return;
        };
        let start_translation = transform.translation();
        let each_rotation = std::f32::consts::PI * 2.0 / peel_skin.count as f32;
        let mut rotation = rand::random_range(0.0..std::f32::consts::PI);
        for _ in 0..peel_skin.count {
            let direction = Vec2::from_angle(rotation);
            commands.spawn((
                Transform::from_translation(
                    Vec3::new(direction.x, direction.y, 0.1) + start_translation,
                ),
                AppleSkin {
                    start_position: start_translation.xy(),
                    normal_speed: peel_skin.start_speed,
                },
            ));
            rotation += each_rotation;
        }
        commands.spawn((ChildOf(event.entity), Sound::new("Apple_Cut")));
    }
}

#[derive(Component, Default)]
#[require(
    Animation::new("Apple_Skin", Vec2::new(40.0, 20.0)),
    enemy::attack::EnemyProjectile,
    movements::ForcedVelocity,
    Collider::rectangle(20.0, 8.0),
    Mass(5.0)
)]
struct AppleSkin {
    start_position: Vec2,
    normal_speed: f32,
}

fn apple_skin_rotate(
    commands: ParallelCommands,
    mut q_skin: Query<(
        Entity,
        &mut movements::ForcedVelocity,
        &mut Transform,
        &mut AppleSkin,
    )>,
    time: Res<Time>,
) {
    const ANGULAR_SPEED: f32 = 3.0;
    let delta = time.delta_secs();
    q_skin.par_iter_mut().for_each(
        |(entity, mut forced_velocity, mut transform, mut apple_skin)| {
            let radius_vector = transform.translation.xy() - apple_skin.start_position;
            let radius_unit = radius_vector.normalize_or_zero();
            apple_skin.normal_speed -= 8.0;
            **forced_velocity = apple_skin.normal_speed * radius_unit;
            let diff =
                Vec2::from_angle(ANGULAR_SPEED * delta).rotate(radius_vector) - radius_vector;
            transform.translation.x += diff.x;
            transform.translation.y += diff.y;
            transform.rotation =
                Quat::from_rotation_z(radius_unit.to_angle() - std::f32::consts::FRAC_PI_2);

            // When passing through origin
            let new_radius =
                transform.translation.xy() + **forced_velocity * delta - apple_skin.start_position;
            if new_radius.dot(radius_unit) <= 0.0 {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                });
            }
        },
    );
}

pub(super) fn change_stage(
    damage: On<health::Damage>,
    mut commands: Commands,
    q_apple: Query<(Entity, &enemy::health::EnemyHealth)>,
    q_manager: Query<(Entity, &ChildOf), With<BehaviorManager>>,
    mut q_peel_skin: Query<(&mut PeelSkin, &mut Behavior, &ChildOf)>,
) {
    let Ok((entity, health)) = q_apple.get(damage.entity) else {
        return;
    };
    let Some((manager_entity, _)) = q_manager.iter().find(|(_, parent)| parent.0 == entity) else {
        return;
    };
    if let Some((mut peel_skin, mut behavior, _)) = q_peel_skin
        .iter_mut()
        .find(|(.., parent)| parent.0 == manager_entity)
    {
        if health.value <= 1000 && peel_skin.count != 8 {
            info!("Apple stage 3 begins");
            peel_skin.count = 8;
            behavior.default_weight = 1.0;
        }
    } else if health.value <= 2000 {
        info!("Apple stage 2 begins");
        commands
            .spawn((ChildOf(manager_entity), PeelSkin::default()))
            .observe(peel_skin_start)
            .observe(peel_skin_respond);
    };
}
