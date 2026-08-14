use super::*;
use enemy::behavior::*;

pub(super) struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(BREAD_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(add_bread_observers)));
        });
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct BreadBehaviors;

fn add_bread_observers(
    _event: On<entry::BreadStart>,
    mut commands: Commands,
    q_bread: Query<Entity, With<BreadBoss>>,
) {
    for entity in q_bread.iter() {
        commands.entity(entity).observe(update_stage);
    }
}

fn update_stage(
    event: On<health::Damage>,
    mut q_bread: Query<(&mut BreadBoss, &enemy::health::EnemyHealth)>,
    mut commands: Commands,
) {
    let Ok((mut bread, health)) = q_bread.get_mut(event.entity) else {
        return;
    };
    let stage: u8 = if health.value > 3800 {
        1
    } else if health.value > 3000 {
        2
    } else if health.value > 1300 {
        3
    } else {
        4
    };
    if stage != bread.stage {
        bread.stage = stage;
        commands.trigger(BreadNextStage(stage));
    }
}
