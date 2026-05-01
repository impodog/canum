use crate::prelude::*;
use canum_res::sound::Sound;

pub(super) struct PlayerAttackPlugin;

impl Plugin for PlayerAttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, (init_weapons,));
        app.world_mut()
            .register_component_hooks::<WeaponSoundCue>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .entity(entity)
                    .observe(sound_weapon_cue_start)
                    .observe(sound_weapon_cue_end);
            });
        app.add_observer(propagate_attack_down)
            .add_observer(propagate_attack_release_down);
    }
}

/// Calls the player to use attack. How to respond depends on weapon itself.
#[derive(EntityEvent, Debug)]
pub struct Attack {
    pub entity: Entity,
}
/// Informs the player weapon that the player released the attack key.
#[derive(EntityEvent, Debug)]
pub struct AttackRelease {
    pub entity: Entity,
}

fn propagate_attack_down(event: On<Attack>, mut commands: Commands, q_children: Query<&Children>) {
    let Ok(children) = q_children.get(event.entity) else {
        return;
    };
    for child in children {
        commands.trigger(Attack { entity: *child });
    }
}
fn propagate_attack_release_down(
    event: On<AttackRelease>,
    mut commands: Commands,
    q_children: Query<&Children>,
) {
    let Ok(children) = q_children.get(event.entity) else {
        return;
    };
    for child in children {
        commands.trigger(AttackRelease { entity: *child });
    }
}

/// The weapons that the player chooses.
#[derive(Component, Debug, Deref, DerefMut, Default)]
pub struct Weapons(pub Vec<Option<Entity>>);

fn init_weapons(mut q_weapons: Query<&mut Weapons>, save: Res<Save>) {
    for mut weapons in q_weapons.iter_mut() {
        if weapons.is_added() {
            weapons.resize(save.progress.weapon_slots, None);
        }
    }
}

/// Marks a player-spawn projectile.
#[derive(Component)]
#[require(
    crate::projectile::Projectile,
    crate::health::Friendly(true),
    crate::health::ContactDamage,
    Animation
)]
pub struct PlayerProjectile;

/// Marks a `Sound` to be played only when the player holds the weapon key.
#[derive(Component)]
#[require(Sound)]
pub struct WeaponSoundCue;

fn sound_weapon_cue_start(event: On<Attack>, mut q_sound: Query<&mut Sound>) {
    let Ok(mut sound) = q_sound.get_mut(event.entity) else {
        return;
    };
    if sound.paused {
        sound.paused = false;
    }
}
fn sound_weapon_cue_end(event: On<AttackRelease>, mut q_sound: Query<&mut Sound>) {
    let Ok(mut sound) = q_sound.get_mut(event.entity) else {
        return;
    };
    if !sound.paused {
        sound.paused = true;
    }
}
