use crate::health::*;
use crate::prelude::*;

pub(super) struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<EnemyHealth>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(enemy_take_damage)
                    .observe(enemy_set_defeat_to_win);
            });
        app.world_mut()
            .register_component_hooks::<EnemySensor>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(sensor_propagate_damage);
            });
        app.world_mut()
            .register_component_hooks::<DamageSound>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(play_damage_sound);
            });
        app.add_systems(FixedPreUpdate, reset_damage_sound_continuous);
    }
}

/// Informs itself that it has been defeated, and effects may play.
#[derive(EntityEvent, Debug)]
pub struct EnemyDefeated {
    pub entity: Entity,
}

/// Basic enemy's health, takes damages unmodified from player projectiles.
#[derive(Component, Default, Debug, Clone)]
#[require(Shields)]
pub struct EnemyHealth {
    pub value: i32,
}
impl EnemyHealth {
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

/// Marks a collider to propagate its taken damage to the parent enemy entity, but not interact with the player.
#[derive(Component, Default)]
#[require(Collider, Transform, Friendly(false), Sensor)]
pub struct EnemySensor;

fn sensor_propagate_damage(event: On<Damage>, q_parent: Query<&ChildOf>, mut commands: Commands) {
    let Ok(parent) = q_parent.get(event.entity) else {
        return;
    };
    commands.trigger(Damage {
        entity: parent.0,
        ..*event
    });
}

fn enemy_take_damage(
    event: On<Damage>,
    mut q_health: Query<(&mut EnemyHealth, &mut Shields)>,
    mut commands: Commands,
) -> Result<()> {
    let (mut health, mut shields) = q_health.get_mut(event.entity)?;
    let damage = shields.take_damage(event.value, event.order);
    if damage <= 0 {
        return Ok(());
    }
    health.value = health.value.saturating_sub(damage);
    if health.value <= 0 && health.value != i32::MIN {
        // Prevents multiple death events
        health.value = i32::MIN;
        commands.trigger(EnemyDefeated {
            entity: event.entity,
        });
    }
    Ok(())
}

fn enemy_set_defeat_to_win(
    event: On<EnemyDefeated>,
    mut q_defeat_to_win: Query<&mut crate::player::victory::DefeatToWin>,
) {
    let Ok(mut defeat_to_win) = q_defeat_to_win.get_mut(event.entity) else {
        return;
    };
    defeat_to_win.defeated = true;
}

#[derive(Component, Default, Debug)]
#[require(DamageSoundContinuous)]
pub struct DamageSound(pub String);
impl DamageSound {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

#[derive(Component, Default, Debug)]
struct DamageSoundContinuous {
    flag: u8,
    child: Option<Entity>,
}

fn reset_damage_sound_continuous(
    commands: ParallelCommands,
    mut q_damage_sound: Query<&mut DamageSoundContinuous>,
) {
    q_damage_sound.par_iter_mut().for_each(|mut continuous| {
        if continuous.flag == 0
            && let Some(child) = continuous.child
        {
            commands.command_scope(|mut commands| {
                commands.entity(child).despawn();
            });
            continuous.child = None;
        }
        continuous.flag = continuous.flag.saturating_sub(1);
    });
}

fn play_damage_sound(
    event: On<Damage>,
    mut commands: Commands,
    mut q_damage_sound: Query<(&DamageSound, &mut DamageSoundContinuous)>,
) {
    let Ok((damage_sound, mut continuous)) = q_damage_sound.get_mut(event.entity) else {
        return;
    };
    continuous.flag = 15;
    if continuous.child.is_none() {
        let child = commands
            .spawn(canum_res::sound::Sound::new(damage_sound.0.clone()))
            .id();
        continuous.child = Some(child);
    }
}
