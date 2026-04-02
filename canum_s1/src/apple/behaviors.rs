use std::time::Duration;

use canum_play::enemy::behavior::*;
use canum_play::prelude::*;

pub(super) struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<AppleBehaviors>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .spawn((ChildOf(entity), MoveCloser))
                    .observe(move_closer)
                    .observe(move_closer_end);
            });
    }
}
#[derive(Component, Debug, Clone)]
#[require(BehaviorManager::new(0.02))]
pub struct AppleBehaviors;

#[derive(Component, Debug, Clone, Default)]
#[require(Behavior::new("Apple_MoveCloser", 1.0, ["Displacement"]), BaseFartherBetter::new(50.0, 100.0))]
struct MoveCloser;
fn move_closer(
    event: On<BehaveStart>,
    mut commands: Commands,
    player: Res<player::RandomPlayer>,
    q_transform: Query<&Transform>,
) -> Result<()> {
    let target_transform = q_transform.get(event.target)?;
    let player_transform = q_transform.get(player.0)?;
    let displace = player_transform.translation.xy() - target_transform.translation.xy();
    commands
        .entity(event.target)
        .insert(enemy::movements::Displacement {
            displace: displace * 0.9,
            duration: Duration::from_secs_f32(2.0),
            notify: Some(event.entity),
        });
    Ok(())
}
fn move_closer_end(event: On<enemy::movements::DisplacementComplete>, mut commands: Commands) {
    commands.trigger(BehaveEnd {
        entity: event.entity,
        cooldown: Duration::from_secs_f32(0.1),
    });
}
