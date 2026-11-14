use super::*;
use canum_tool::weapon::laser::*;

pub(super) struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<Laser>()
            .on_add(laser_hook);
        app.add_systems(FixedUpdate, laser_linger);
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[require(Weapon, Transform, Visibility, LaserArgs)]
pub struct Laser {
    pub charge_time: f32,
    pub linger_time: f32,
    pub damage: math::ApproxFloat,
    pub order: u8,
}
#[derive(Component, Default)]
struct LaserArgs {
    charge: Timer,
}
#[derive(Component, Default)]
struct LaserLinger {
    pub time: Timer,
    pub rotation_base: f32,
    pub rotation_limit: f32,
}
impl LaserLinger {
    const ROTATION_LIMIT_BASE: f32 = std::f32::consts::PI / 11.0;
}
impl Default for Laser {
    fn default() -> Self {
        Self {
            charge_time: 1.0,
            linger_time: 0.4,
            damage: math::ApproxFloat::from(132),
            order: consts::order::PLAYER_PROJ_STRONG,
        }
    }
}

fn laser_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let args = *world.get::<Laser>(entity).unwrap();
    world
        .commands()
        .entity(entity)
        .insert(LaserArgs {
            charge: Timer::from_seconds(args.charge_time, TimerMode::Once),
        })
        .observe(laser_charge)
        .observe(laser_release);
}

fn laser_charge(
    event: On<Attack>,
    mut q_laser: Query<&mut LaserArgs>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let Ok(mut args) = q_laser.get_mut(event.entity) else {
        return;
    };
    if !args.charge.is_finished() {
        args.charge.tick(time.delta());
        commands.trigger(crate::misc::weapon_indic::UpdateChargeIndicatorByWeapon {
            entity: event.entity,
            value: args.charge.fraction(),
        });
    }
}

fn laser_release(
    event: On<AttackRelease>,
    mut q_laser: Query<(&Laser, &mut LaserArgs, &ChildOf)>,
    q_player: Query<(&player::PlayerShoot, &LinearVelocity)>,
    mut commands: Commands,
) {
    const COLLIDER_SIZE: Vec2 = vec2(32.0, 12.0);
    const IMAGE_SIZE: Vec2 = vec2(32.0, 32.0);
    let Ok((laser, mut args, parent)) = q_laser.get_mut(event.entity) else {
        return;
    };
    if args.charge.is_finished() {
        let Ok((shoot, linear_velocity)) = q_player.get(parent.0) else {
            return;
        };
        let add = 5.0 * Vec2::from_angle(shoot.0);
        let transform = Transform::from_translation(vec3(add.x, add.y, -10.0))
            .with_rotation(Quat::from_rotation_z(shoot.0));
        commands.spawn((
            SessionOnly,
            transform,
            LaserLike {
                base_direction: Dir2::from_xy_unchecked(1.0, 0.0),
                middle: SpriteSheet::new("Player_Laser_Middle"),
                terminal: SpriteSheet::new("Player_Laser_Terminal"),
                length: COLLIDER_SIZE.x,
                width: IMAGE_SIZE.y,
                playback_interval: 0.1,
                collide_width: COLLIDER_SIZE.y,
                ignore_layer: LaserLayer::LASER_PLAYER | LaserLayer::LASER_PROJECTILE,
                animation_kind: default(),
            },
            LaserLinger {
                time: Timer::from_seconds(laser.linger_time, TimerMode::Once),
                rotation_base: shoot.0,
                rotation_limit: LaserLinger::ROTATION_LIMIT_BASE
                    * (1.0 - linear_velocity.length() / 200.0).max(0.5),
            },
            canum_fx::transform::Follow::new(parent.0)
                .with_follow_behavior(canum_fx::transform::FollowBehavior::NoRotation),
            health::ContactDamage {
                value: laser.damage.sample(),
                projectile: true,
                order: laser.order,
            },
            health::Friendly::FRIENDLY,
        ));
        commands.spawn(Sound::new("Player_Laser"));
    }
    args.charge.reset();
    commands.trigger(crate::misc::weapon_indic::UpdateChargeIndicatorByWeapon {
        entity: event.entity,
        value: 0.0,
    });
}

fn laser_linger(
    mut q_linger: Query<(
        Entity,
        &mut LaserLinger,
        &mut Transform,
        &canum_fx::transform::Follow,
    )>,
    q_player_shoot: Query<&player::PlayerShoot>,
    commands: ParallelCommands,
    time: Res<Time>,
) {
    q_linger
        .par_iter_mut()
        .for_each(|(entity, mut linger, mut transform, follow)| {
            if linger.time.tick(time.delta()).just_finished() {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).try_despawn();
                });
            } else {
                let Ok(player_shoot) = q_player_shoot.get(follow.target) else {
                    return;
                };
                let angle_diff = normalize_angle_signed(player_shoot.0 - linger.rotation_base);
                let rotation = angle_diff.clamp(-linger.rotation_limit, linger.rotation_limit)
                    + linger.rotation_base;
                transform.rotation = Quat::from_rotation_z(rotation);
            }
        });
}
