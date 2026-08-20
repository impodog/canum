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
        app.world_mut().register_component_hooks::<Mantis>().on_add(
            |mut world, HookContext { entity, .. }| {
                world.commands().spawn((
                    ChildOf(entity),
                    MantisBehaviors,
                    children![AdjustPosition],
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
    Animation::new("Projected_Mantis_Static", vec2(32.0, 32.0)),
    enemy::health::EnemyHealth::new(300),
    movements::AutoFlip::FLIP_RIGHT
)]
pub struct Mantis;

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
struct MantisBehaviors;

#[derive(Component, Default)]
#[require(Behavior::new("Projected_Mantis_AdjustPosition", 0.5, ["Mantis"]))]
struct AdjustPosition;

fn adjust_position(
    event: On<BehaveStart>,
    mut commands: Commands,
    primary_player: Res<player::PrimaryPlayer>,
    q_transform: Query<&GlobalTransform>,
) {
    let Ok(player_transform) = q_transform.get(primary_player.0) else {
        return;
    };
    let player_position = player_transform.translation().xy();
    let Ok(transform) = q_transform.get(event.entity) else {
        return;
    };
    let position = transform.translation().xy();
    let direction = (position - player_position).normalize_or_zero();
    let target = direction * rand_normal(100.0, 10.0) + player_position;
    let displace = target - position;
    commands.spawn((
        ChildOf(event.target),
        enemy::movements::Displacement {
            curve: |x| x,
            displace,
            duration: Duration::from_secs_f32((displace.length() / 100.0).min(5.0)),
            notify: Some(event.entity),
        },
    ));
}
fn adjust_position_end(event: On<enemy::movements::DisplacementComplete>, mut commands: Commands) {
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(rand_normal(2.0, 0.2)),
        occupies: occupies![],
    });
}

#[derive(Component, Default)]
#[require(Behavior::new("Projected_Mantis_Slash", 1.0, ["Mantis"]), BaseByDistance::new(100.0, 15.0))]
struct Slash;
