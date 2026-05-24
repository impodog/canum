use super::*;

pub(super) struct DefeatPlugin;

impl Plugin for DefeatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPostUpdate,
            test_if_defeated.run_if(in_state(RUNWAY_STATE.clone())),
        );
    }
}

fn test_if_defeated(
    q_lion: Query<&GlobalTransform, With<RunwayLion>>,
    mut q_lion_animation: Query<&mut Animation, With<RunwayLion>>,
    mut q_boss: Query<(Entity, &mut player::victory::DefeatToWin), With<RunwayBoss>>,
    mut commands: Commands,
    mut speed: ResMut<running::RollingSpeed>,
    q_behavior: Query<Entity, With<enemy::behavior::BehaviorManager>>,
) {
    let Ok(transform) = q_lion.single() else {
        return;
    };
    let translation = transform.translation();
    if translation.y - CONFIG.display.half_virtual_size.1 >= -1.0 {
        let Ok((entity, mut defeated)) = q_boss.single_mut() else {
            return;
        };
        defeated.defeated = true;
        commands
            .entity(entity)
            .remove::<player::victory::DefeatToWin>();
        for manager in q_behavior.iter() {
            commands.entity(manager).despawn();
        }
        let Ok(mut animation) = q_lion_animation.single_mut() else {
            return;
        };
        animation.replace("Runway_Lion_Defeated", false, None);
        speed.0 = 0.0;
    }
}
