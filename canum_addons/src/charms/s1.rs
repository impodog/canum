use std::collections::btree_map::Entry;

use super::*;

pub(super) struct S1Plugin;

impl Plugin for S1Plugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_exoskeleton_ui)
            .add_observer(add_exoskeleton);
        app.add_systems(
            FixedUpdate,
            (exoskeleton_increase, exoskeleton_protect).chain(),
        );
    }
}

#[derive(Component, Default)]
#[require(
    Sensor,
    Collider::circle(64.0),
    projectile::NoDisposeProjectile,
    CollidingEntities
)]
pub struct Exoskeleton;
fn add_exoskeleton(event: On<setup::StartSessionAction>, save: Res<Save>, mut commands: Commands) {
    for _ in save
        .progress
        .selected_effects
        .range_starting_with("Exoskeleton")
    {
        commands.spawn((
            ChildOf(event.player_entity),
            Exoskeleton,
            Transform::from_translation(vec3(0.0, 0.0, -1.1)),
            Animation::new("Circle64", vec2(64.0, 64.0)).with_color(Color::BLACK.with_alpha(0.4)),
        ));
    }
}

const EXOSKELETON_CONSUMPTION: f32 = 1.0;
const EXOSKELETON_INCREASE_PER_SEC: f32 = 0.1;
const EXOSKELETON_INVINC_TIME: f32 = 1.0;

#[derive(Component, Default, Clone, Copy)]
#[require(Node)]
struct ExoskeletonUi {
    amount: f32,
    max: f32,
}
fn spawn_exoskeleton_ui(
    event: On<setup::StartSessionAddedUi>,
    q_children: Query<&Children>,
    q_exoskeleton: Query<(), With<Exoskeleton>>,
    left_top: Single<Entity, With<canum_ui::TopLeft>>,
    mut commands: Commands,
) {
    let Ok(children) = q_children.get(event.player_entity) else {
        return;
    };
    let number = children
        .iter()
        .filter(|child| q_exoskeleton.get(*child).is_ok())
        .count();
    if number != 0 {
        let max = (number as f32).sqrt();
        commands.spawn((
            ChildOf(left_top.entity()),
            ExoskeletonUi { amount: 0.0, max },
            canum_ui::bar::health_bar(
                Color::Srgba(Srgba::hex("#0b73ff").unwrap()),
                Color::Srgba(Srgba::hex("#07101d").unwrap()),
                canum_ui::bar::HealthBar {
                    current: 0.0,
                    total: max,
                    width: max * 100.0,
                    ..default()
                },
            ),
        ));
    }
}

fn exoskeleton_increase(
    mut q_exoskeleton: Single<(&mut ExoskeletonUi, &mut canum_ui::bar::HealthBar)>,
    q_sensor: Query<&CollidingEntities, With<Exoskeleton>>,
    q_projectile: Query<(), With<projectile::Projectile>>,
    q_friendly: Query<&health::Friendly>,
    time: Res<Time>,
) {
    let (ref mut exoskeleton, ref mut bar) = *q_exoskeleton;
    let mut incr = 0.0;
    for colliding_entities in q_sensor.iter() {
        for entity in colliding_entities.iter() {
            if q_projectile.get(*entity).is_ok()
                && let Ok(friendly) = q_friendly.get(*entity)
                && !friendly.0
            {
                incr += EXOSKELETON_INCREASE_PER_SEC * time.delta_secs();
            }
        }
    }
    exoskeleton.amount = (exoskeleton.amount + incr).min(exoskeleton.max);
    bar.current = exoskeleton.amount;
}

fn exoskeleton_protect(
    mut exoskeleton: Single<&mut ExoskeletonUi>,
    mut q_exoskeleton_animation: Query<&mut Animation, With<Exoskeleton>>,
    q_children: Query<&Children>,
    mut q_shields: Query<(Entity, &mut health::Shields), With<player::Player>>,
    mut commands: Commands,
) {
    if exoskeleton.amount < EXOSKELETON_CONSUMPTION {
        return;
    }
    for (entity, mut shields) in q_shields.iter_mut() {
        let entry = shields.entry(consts::order::EXOSKELETON);
        match entry {
            Entry::Vacant(vacant) => {
                // The exoskeleton is charged, and needs actual effect.
                vacant.insert(i32::MAX);
                // Update indicator color
                let Ok(children) = q_children.get(entity) else {
                    return;
                };
                for child in children.iter() {
                    if let Ok(mut animation) = q_exoskeleton_animation.get_mut(child) {
                        animation.color = Color::linear_rgba(0.0, 1.0, 1.0, 0.8);
                    }
                }
            }
            Entry::Occupied(occupied) => {
                // The exoskeleton is consumed.
                if *occupied.get() != i32::MAX {
                    occupied.remove_entry();
                    exoskeleton.amount = (exoskeleton.amount - EXOSKELETON_CONSUMPTION).max(0.0);
                    commands.spawn((
                        ChildOf(entity),
                        health::InvincibilityTimer::new(
                            EXOSKELETON_INVINC_TIME,
                            consts::order::HEALTH_INVINC,
                        ),
                    ));

                    commands.spawn(Sound::new("ShieldBreak"));

                    // Restore indicator color.
                    let Ok(children) = q_children.get(entity) else {
                        return;
                    };
                    for child in children.iter() {
                        if let Ok(mut animation) = q_exoskeleton_animation.get_mut(child) {
                            animation.color = Color::linear_rgba(0.0, 0.0, 0.0, 0.4);
                        }
                    }
                }
            }
        }
    }
}
