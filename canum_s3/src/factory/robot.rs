use super::*;
use enemy::behavior::*;

pub(super) struct RobotPlugin;

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (dash_away_multiplier,).in_set(BehaviorProcess::Calc),
        );
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct RobotBehaviors;

fn robot_behaviors_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world
        .commands()
        .spawn((ChildOf(entity), DashAway))
        .observe(start_dash_away);
}

#[derive(Component, Default)]
#[require(Behavior::new("Factory_Robot_DashAway", 0.35, ["Move", "DashAway"]), MultiplierManual)]
pub struct DashAway;

fn dash_away_multiplier(
    mut q_dash_away: Query<(&mut MultiplierManual, &GlobalTransform), With<DashAway>>,
    q_projectile: Query<(&GlobalTransform, &health::ContactDamage), With<health::PlayerRelated>>,
    attack_info: Res<player::attack::PlayerAttackInfo>,
) {
    // Only update occasionally.
    if rand_bool(0.8) {
        return;
    }
    let (position, damage) = q_projectile.iter().fold((Vec2::ZERO, 0.0f32), |prev, new| {
        let value = new.1.value as f32 * 0.01;
        (prev.0 + new.0.translation().xy() * value, prev.1 + value)
    });
    let average = position * damage.recip().min(1e6);
    for (mut multiplier, global_transform) in q_dash_away.iter_mut() {
        let position = global_transform.translation().xy();
        let distance = average.distance(position);
        let weight = ((distance + 50.0).recip() * 50.0).clamp(0.0, 1.0);
        let weight = (weight + attack_info.charge_ratio).clamp(0.0, 1.0);
        multiplier.0 = weight;
    }
}

fn start_dash_away(
    event: On<BehaveStart>,
    mut q_animation: Query<&mut Animation>,
    mut commands: Commands,
    player: Res<player::PrimaryPlayer>,
    q_transform: Query<&GlobalTransform>,
) {
    let Ok(mut animation) = q_animation.get_mut(event.target) else {
        return;
    };
    let Ok(transform) = q_transform.get(event.entity) else {
        return;
    };
    let Ok(player_transform) = q_transform.get(player.0) else {
        return;
    };
    animation.replace("Factory_Robot_Dash", false, None);
    let position = transform.translation().xy();
    let player_position = player_transform.translation().xy();
    let diff = (position - player_position).normalize_or(vec2(1.0, 0.0));
    let distance = rand_range(80.0f32..120.0);
    let angle = std::f32::consts::FRAC_PI_2 * rand_range(0.9..1.1) * rand_sign();
    let displace = diff.rotate(Vec2::from_angle(angle)) * distance;
    commands.spawn((
        ChildOf(event.target),
        enemy::movements::Displacement {
            curve: canum_fx::quadratic_curve!(1.0, 0.2),
            displace,
            duration: Duration::from_secs_f32(distance / 300.0),
            notify: Some(event.entity),
        },
    ));
}
