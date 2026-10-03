use super::*;

pub(super) struct PlayerStatsPlugin;

impl Plugin for PlayerStatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DynamicPlayerArguments>();
        app.add_observer(init_player_arguments)
            .add_observer(query_player_effects)
            .add_observer(apply_effects_to_player);
        app.add_systems(
            FixedPreUpdate,
            apply_dynamic_player_damage.before(health::DamageApplicationSet),
        );
    }
}

#[derive(Resource, Debug)]
pub struct PlayerArguments {
    pub speed_addition: f32,
}
impl Default for PlayerArguments {
    fn default() -> Self {
        Self {
            speed_addition: 0.0,
        }
    }
}

fn init_player_arguments(_event: On<setup::StartSessionFirst>, mut commands: Commands) {
    commands.insert_resource(PlayerArguments::default());
    commands.insert_resource(DynamicPlayerArguments::default());
}

fn query_player_effects(
    _event: On<setup::StartSessionMiddle>,
    save: Res<Save>,
    mut args: ResMut<PlayerArguments>,
) {
    for effect in save
        .progress
        .selected_effects
        .range_starting_with("MoveSpeed+%")
    {
        if let Some(value) = effect.strip_prefix("MoveSpeed+%") {
            match value.parse::<i32>() {
                Ok(percent) => {
                    args.speed_addition += percent as f32 * 0.01;
                }
                Err(err) => {
                    error!("Invalid player move speed addition effect: {value}, {err}");
                }
            }
        }
    }
}

fn apply_effects_to_player(
    _event: On<setup::StartSessionAction>,
    mut player: Query<&mut player::PlayerMoveSpeed>,
    args: Res<PlayerArguments>,
) {
    let total_move_speed_multiplier = 1.0 + args.speed_addition;
    for mut move_speed in player.iter_mut() {
        move_speed.0 *= total_move_speed_multiplier;
    }
}

/// These arguments are re-applied each fixed frame. They are updated by other effects dynamically.
#[derive(Resource)]
pub struct DynamicPlayerArguments {
    pub damage_addition: f64,
    pub damage_multiplier: f64,
}
impl Default for DynamicPlayerArguments {
    fn default() -> Self {
        Self {
            damage_addition: 0.0,
            damage_multiplier: 1.0,
        }
    }
}

fn apply_dynamic_player_damage(
    mut q_proj: Query<
        &mut health::ContactDamageModifier,
        (With<health::PlayerRelated>, Added<health::ContactDamage>),
    >,
    args: Res<DynamicPlayerArguments>,
) {
    q_proj.par_iter_mut().for_each(|mut modifier| {
        modifier.multiplier_addition += args.damage_addition;
        modifier.multiplier_multiply *= args.damage_multiplier;
    })
}
