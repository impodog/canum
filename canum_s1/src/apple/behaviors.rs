use canum_play::enemy::behavior::*;
use canum_play::prelude::*;

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
            (apple_slice_revolve).run_if(in_state(super::APPLE_STATE.clone())),
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
    });
}

#[derive(Component, Default)]
#[require(
    Behavior::new("Apple_ThrowSlice", 2.0, ["Animation", "Projectile"]),
    BaseByDistance::new(200.0, 100.0)
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
            LinearVelocity(direction * 200.0),
        ));
        commands.trigger(BehaveEnd {
            entity: event.entity,
            // Cooldown exists in
            cooldown: Duration::from_secs_f32(rand_normal(1.5, 0.3)),
        });
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
