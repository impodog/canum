use super::*;

pub(super) struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<Laser>()
            .on_add(laser_hook);
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[require(Transform, Visibility, LaserArgs)]
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
}
impl Default for Laser {
    fn default() -> Self {
        Self {
            charge_time: 1.2,
            linger_time: 0.5,
            damage: math::ApproxFloat::from(110),
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
    args.charge.tick(time.delta());
    commands.trigger(crate::misc::weapon_indic::UpdateChargeIndicatorByWeapon {
        entity: event.entity,
        value: args.charge.fraction(),
    });
}

fn laser_release(
    event: On<AttackRelease>,
    mut q_laser: Query<&mut LaserArgs>,
    mut commands: Commands,
) {
    let Ok(mut args) = q_laser.get_mut(event.entity) else {
        return;
    };
    args.charge.reset();
    commands.trigger(crate::misc::weapon_indic::UpdateChargeIndicatorByWeapon {
        entity: event.entity,
        value: 0.0,
    });
}
