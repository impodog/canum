use super::*;
use std::collections::HashSet;

pub(super) struct WeaponIndicPlugin;

impl Plugin for WeaponIndicPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_aim_indic)
            .add_observer(spawn_charge_indic)
            .add_observer(update_player_charge_indic);
        app.add_systems(Startup, init_weapon_requires_charging);
        app.add_systems(FixedUpdate, update_charge_indic);
        app.add_systems(Update, move_charge_indic);
    }
}

/// Indicator for the player's aiming direction.
#[derive(Component, Default)]
#[require(
    Animation::new("Player_Indic_Aim", vec2(32.0, 32.0)),
    Transform::from_translation(vec3(0.0, 16.0 + 16.0, -2.5))
)]
pub struct AimIndic;

fn spawn_aim_indic(event: On<setup::StartSessionAction>, mut commands: Commands, save: Res<Save>) {
    if save.options.show_aim {
        commands.spawn((ChildOf(event.player_entity), AimIndic));
    }
}

/// A ring that indicates the charge weapon's status. You need to update by querying `ChangeIndicValue`.
#[derive(Component, Default)]
#[require(SpriteSheet::new("Player_Indic_Charge"), ChargeIndicValue, ChargeIndicParent, Sprite {custom_size: Some(vec2(16.0, 16.0)), ..default()})]
pub struct ChargeIndic;
#[derive(Component, Default)]
pub struct ChargeIndicValue(pub f32);

/// A component of the player, marks the charge indicator child.
#[derive(Component)]
pub struct ChargeIndicChild(pub Entity);
#[derive(Component, Default)]
pub struct ChargeIndicParent(pub Option<Entity>);

fn update_charge_indic(
    mut q_charge_indic: Query<
        (&ChargeIndicValue, &mut SpriteSheetIndex, &SpriteSheetMeta),
        Changed<ChargeIndicValue>,
    >,
) {
    q_charge_indic
        .par_iter_mut()
        .for_each(|(value, mut index, meta)| {
            let Some(meta) = meta.0 else {
                return;
            };
            let new_index = (value.0 * meta.count as f32).floor() as u32;
            let new_index = new_index.min(meta.count.saturating_sub(1)) as usize;
            if index.0 != new_index {
                index.0 = new_index;
            }
        });
}

#[derive(Resource, Default, Deref)]
struct WeaponRequiresCharging(HashSet<String>);

fn init_weapon_requires_charging(mut commands: Commands) {
    if let Some(weapons) = CONFIG.values.custom.get("Weapon_RequiresCharging")
        && let Ok(weapons) = weapons.clone().into_rust::<HashSet<String>>()
    {
        commands.insert_resource(WeaponRequiresCharging(weapons));
    } else {
        error!(
            "Custom value \"Weapon_RequiresCharging\" is not the correct format. Expected HashSet<String>."
        );
        commands.init_resource::<WeaponRequiresCharging>();
    }
}

fn spawn_charge_indic(
    event: On<setup::StartSessionAction>,
    save: Res<Save>,
    weapons: Res<WeaponRequiresCharging>,
    mut commands: Commands,
) {
    let ok = save
        .progress
        .selected_weapons
        .iter()
        .any(|weapon| weapons.contains(weapon));
    if ok {
        let child = commands
            .spawn((ChargeIndicParent(Some(event.player_entity)), ChargeIndic))
            .id();
        commands
            .entity(event.player_entity)
            .insert(ChargeIndicChild(child));
    }
}

const RELATIVE_TRANSFORM: Vec3 = vec3(16.0, 32.0, -4.0);

fn move_charge_indic(
    mut q_indic: Query<(&mut Transform, &ChargeIndicParent)>,
    q_global_transform: Query<&GlobalTransform>,
) {
    for (mut transform, parent) in q_indic.iter_mut() {
        let Some(parent) = parent.0 else {
            return;
        };
        let Ok(parent_transform) = q_global_transform.get(parent) else {
            return;
        };
        let position = RELATIVE_TRANSFORM.xy().rotate(Vec2::from_angle(
            parent_transform.rotation().to_euler(EulerRot::XYZ).2,
        ));
        let translation =
            parent_transform.translation() + vec3(position.x, position.y, RELATIVE_TRANSFORM.z);
        transform.translation = translation;
    }
}

#[derive(Event, Debug)]
pub struct UpdateChargeIndicatorByWeapon {
    pub entity: Entity,
    pub value: f32,
}

fn update_player_charge_indic(
    event: On<UpdateChargeIndicatorByWeapon>,
    mut q_indic: Query<&mut crate::misc::weapon_indic::ChargeIndicValue>,
    q_parent: Query<&ChildOf>,
    q_child: Query<&crate::misc::weapon_indic::ChargeIndicChild>,
) {
    let Ok(parent) = q_parent.get(event.entity) else {
        return;
    };
    let Ok(child) = q_child.get(parent.0) else {
        return;
    };
    let Ok(mut indic) = q_indic.get_mut(child.0) else {
        return;
    };
    indic.0 = event.value;
}
