use bevy::ecs::lifecycle::HookContext;

use crate::health::*;
use crate::prelude::*;

pub(super) struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<EnemyHealth>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(enemy_take_damage);
            });
        app.world_mut()
            .register_component_hooks::<DamageSound>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(play_damage_sound);
            });
        app.add_systems(FixedPreUpdate, reset_damage_sound_continuous);
    }
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

fn enemy_take_damage(
    event: On<Damage>,
    mut q_health: Query<(&mut EnemyHealth, &mut Shields)>,
) -> Result<()> {
    let (mut health, mut shields) = q_health.get_mut(event.entity)?;
    let damage = shields.take_damage(event.value, event.order);
    if damage <= 0 {
        return Ok(());
    }
    health.value = health.value.saturating_sub(damage);
    Ok(())
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
