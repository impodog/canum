use super::*;

pub(super) struct MantisPlugin;

impl Plugin for MantisPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<AdjustPosition>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(adjust_position)
                    .observe(adjust_position_end);
            });
        app.world_mut()
            .register_component_hooks::<SlashResidue>()
            .on_add(slash_residue_hook);
        app.world_mut()
            .register_component_hooks::<DoSlash>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(do_slash)
                    .observe(do_slash_end);
            });
        app.world_mut().register_component_hooks::<Mantis>().on_add(
            |mut world, HookContext { entity, .. }| {
                world.commands().spawn((
                    ChildOf(entity),
                    MantisBehaviors,
                    children![AdjustPosition, DoSlash],
                ));
                world.commands().spawn((
                    ChildOf(entity),
                    enemy::health::EnemySensor,
                    Collider::rectangle(20.0, 32.0),
                ));
            },
        );
    }
}

#[derive(Component, Default)]
#[require(
    ProjectedEnemy,
    Collider::rectangle(10.0, 20.0),
    Animation::new("Projected_Mantis_Static", vec2(64.0, 64.0)),
    enemy::health::EnemyHealth::new(300),
    movements::AutoFlip::FLIP_RIGHT
)]
pub struct Mantis;

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
struct MantisBehaviors;

#[derive(Component, Default)]
#[require(Behavior::new("Projected_Mantis_AdjustPosition", 1.0, ["Mantis"]))]
struct AdjustPosition;

fn adjust_position(
    event: On<BehaveStart>,
    mut commands: Commands,
    primary_player: Option<Res<player::PrimaryPlayer>>,
    q_transform: Query<&GlobalTransform>,
) {
    let Some(primary_player) = primary_player else {
        return;
    };
    let Ok(player_transform) = q_transform.get(primary_player.0) else {
        return;
    };
    let player_position = player_transform.translation().xy();
    let Ok(transform) = q_transform.get(event.entity) else {
        return;
    };
    let position = transform.translation().xy();
    let direction = (position - player_position).normalize_or_zero();
    let target = direction * rand_normal(90.0, 5.0) + player_position;
    let displace = target - position;
    commands.spawn((
        ChildOf(event.target),
        enemy::movements::Displacement {
            curve: |x| x,
            displace,
            duration: Duration::from_secs_f32((displace.length() / 90.0).min(5.0)),
            notify: Some(event.entity),
        },
    ));
}
fn adjust_position_end(event: On<enemy::movements::DisplacementComplete>, mut commands: Commands) {
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(rand_normal(0.5, 0.05)),
        occupies: occupies![],
    });
}

#[derive(Component, Default)]
#[require(Behavior::new("Projected_Mantis_Slash", 1.0, ["Mantis"]), BaseByDistance::new(90.0, 10.0))]
struct DoSlash;

#[derive(Component, Default)]
#[require(Animation, Collider, health::ContactDamage {value: 50, projectile: true, order: consts::order::ENEMY_PROJ})]
struct SlashResidue;

fn slash_residue_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world
        .commands()
        .entity(entity)
        .insert(
            Animation::new("Projected_Mantis_SlashResidue", vec2(48.0, 48.0))
                .with_inform(AnimationInform {
                    entity,
                    index: vec![0, 1, 2],
                })
                .once_then_despawn(),
        )
        .observe(slash_residue_change_hitbox);
}
fn slash_residue_change_hitbox(event: On<AnimationComplete>, mut commands: Commands) {
    let length: f32 = match event.index {
        0 => 15.0,
        1 => 30.0,
        _ => 48.0,
    };
    commands
        .entity(event.entity)
        .insert(Collider::compound(vec![(
            vec2(0.0, 16.0 - length * 0.5),
            0.0,
            Collider::rectangle(15.0, length),
        )]));
}

fn do_slash(
    event: On<BehaveStart>,
    player: Option<Res<player::PrimaryPlayer>>,
    q_transform: Query<&GlobalTransform>,
    mut q_sprite: Query<(&mut Sprite, &mut Animation)>,
    mut commands: Commands,
) {
    let Some(player) = player else {
        return;
    };
    let Ok(player_transform) = q_transform.get(player.0) else {
        return;
    };
    let player_position = player_transform.translation().xy();
    let Ok(transform) = q_transform.get(event.entity) else {
        return;
    };
    let position = transform.translation().xy();
    let direction = (player_position - position).normalize_or(vec2(1.0, 0.0));
    let angle = direction.to_angle();
    let left_half = angle.abs() > std::f32::consts::FRAC_PI_2;
    commands.spawn((
        ChildOf(event.target),
        enemy::movements::Displacement {
            curve: |x| QuadraticOutCurve.sample(x).unwrap(),
            displace: direction * rand_normal(20.0, 2.0),
            duration: Duration::from_secs_f32(0.2),
            notify: None,
        },
    ));
    commands.spawn((
        ChildOf(event.target),
        SlashResidue,
        Transform::from_translation(
            vec3(80.0, 0.0, transform.translation().z + 0.5).rotate_z(angle),
        )
        .with_rotation(Quat::from_rotation_z(if left_half {
            angle - angle.signum() * std::f32::consts::PI
        } else {
            angle
        })),
        Sprite {
            flip_x: left_half,
            flip_y: left_half,
            ..default()
        },
    ));
    commands.spawn(Sound::new("Projected_Mantis_Slash"));

    let Ok((mut sprite, mut animation)) = q_sprite.get_mut(event.target) else {
        return;
    };
    if left_half {
        sprite.flip_x = true;
    }
    animation.replace(
        "Projected_Mantis_Slash",
        true,
        Some(AnimationInform {
            entity: event.entity,
            index: vec![usize::MAX],
        }),
    );
}
fn do_slash_end(
    event: On<AnimationComplete>,
    mut q_animation: Query<&mut Animation>,
    mut commands: Commands,
) {
    let Ok(mut animation) = q_animation.get_mut(event.source) else {
        return;
    };
    animation.replace("Projected_Mantis_Static", false, None);
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(5.0),
        occupies: occupies![],
    });
}
