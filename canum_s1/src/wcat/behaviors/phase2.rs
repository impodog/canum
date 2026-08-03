use super::*;

pub(super) struct Phase2Plugin;

impl Plugin for Phase2Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (enter_phase_2, high_lunge_add_observer).run_if(in_state(WCAT_STATE.clone())),
        );
        app.add_systems(
            FixedPostUpdate,
            (high_lunge_smash_accelerate, high_lunge_update_shadow_phase2)
                .run_if(in_state(WCAT_STATE.clone())),
        );
    }
}

#[derive(Component, Default)]
#[require(BehaviorManager::new())]
pub struct WcatPhase2;

#[derive(Component)]
pub struct EnteredPhase2;

#[allow(clippy::type_complexity)]
fn enter_phase_2(
    q_cat: Query<
        (Entity, &enemy::health::EnemyHealth, &Children),
        (With<WcatBoss>, Without<EnteredPhase2>),
    >,
    q_phase1: Query<(), With<phase1::WcatPhase1>>,
    mut commands: Commands,
) {
    for (entity, health, children) in q_cat.iter() {
        if health.value <= 2500 {
            commands.entity(entity).insert(EnteredPhase2);
            for child in children.iter() {
                if q_phase1.get(child).is_ok() {
                    commands.entity(child).despawn();
                }
            }
            commands.spawn((
                ChildOf(entity),
                WcatPhase2,
                children![
                    (
                        phase1::WanderAround,
                        // Now it fights for Velocity
                        Behavior::new("Wcat_WanderAround", 1.0, ["WanderAround", "Velocity"])
                    ),
                    (
                        phase1::HighLunge {
                            time: 0.18,
                            min_time: 0.3,
                            predict_strength: 0.35,
                            high_lunge_y_shift: 100.0,
                            ..default()
                        },
                        Behavior::new("Wcat_HighLunge", 1.0, ["Animation", "Velocity"])
                    ),
                ],
            ));
        }
    }
}

#[derive(EntityEvent, Debug)]
pub struct HighLungeSmash {
    pub entity: Entity,
}

#[derive(Component, Debug)]
pub struct SmashTimers {
    pub waiting: Timer,
    pub timer: Timer,
    pub additional: Timer,
}
impl Default for SmashTimers {
    fn default() -> Self {
        Self {
            waiting: Timer::from_seconds(0.1, TimerMode::Once),
            timer: Timer::from_seconds(SMASH_TIME, TimerMode::Once),
            additional: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
}

const SMASH_TIME: f32 = 0.7;
const SMASH_ACCELERATION: f32 = 400.0;

fn high_lunge_smash(
    event: On<HighLungeSmash>,
    mut commands: Commands,
    mut q_timer: Query<(&ChildOf, &mut movements::PartialVelocity)>,
    mut q_animation: Query<&mut Animation>,
) {
    commands.entity(event.entity).insert(SmashTimers::default());
    let Ok((parent, mut partial_velocity)) = q_timer.get_mut(event.entity) else {
        return;
    };
    **partial_velocity = vec2(0.0, 0.0);
    let Ok(mut animation) = q_animation.get_mut(parent.0) else {
        return;
    };
    animation.replace("Wcat_Smash", false, None);
}

fn high_lunge_add_observer(
    q_timers: Query<Entity, Added<phase1::HighLungeTimers>>,
    mut commands: Commands,
) {
    for entity in q_timers.iter() {
        commands.entity(entity).observe(high_lunge_smash);
    }
}

fn high_lunge_smash_accelerate(
    mut q_smash: Query<(
        Entity,
        &mut SmashTimers,
        &mut movements::PartialVelocity,
        &phase1::HighLungeTimers,
        &ChildOf,
    )>,
    mut q_animation: Query<&mut Animation>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut timers, mut partial_velocity, high_lunge_timers, parent) in q_smash.iter_mut()
    {
        if timers.waiting.is_finished() {
            if timers.timer.is_finished() {
                if timers.additional.tick(time.delta()).just_finished() {
                    commands.entity(entity).despawn();
                    commands.trigger(BehaveEnd {
                        entity: high_lunge_timers.source,
                        cooldown: Duration::from_secs_f32(rand_normal(2.0, 0.3).clamp(1.5, 2.5)),
                        occupies: occupies![("HighLunge", rand::random_range(4.0..5.5))],
                    });
                    let Ok(mut animation) = q_animation.get_mut(parent.0) else {
                        return;
                    };
                    animation.replace("Wcat_Static", false, None);
                }
            } else {
                partial_velocity.y -= SMASH_ACCELERATION * time.delta_secs();
                if timers.timer.tick(time.delta()).just_finished() {
                    partial_velocity.y = 0.0;
                    commands.spawn(Sound::new("Wcat_Smash"));
                    commands.spawn((
                        ChildOf(entity),
                        Animation::new("Wcat_Shockwave", vec2(150.0, 150.0)).once(),
                        Transform::from_translation(vec3(0.0, -10.0, -0.1)),
                        RigidBody::Kinematic,
                        Collider::circle(70.0),
                        health::Friendly::UNFRIENDLY,
                        health::ContactDamage {
                            value: 200,
                            projectile: false,
                            order: consts::order::ENEMY_BOSS,
                        },
                    ));
                }
            }
        } else if timers.waiting.tick(time.delta()).just_finished() {
            let total_time = timers.timer.duration().as_secs_f32();
            let initial_speed = -100.0 / total_time + SMASH_ACCELERATION * 0.5 * total_time;
            **partial_velocity = vec2(0.0, initial_speed);
        }
    }
}

#[derive(Component, Default)]
pub struct HighLungeShadowPhase2;

fn high_lunge_update_shadow_phase2(
    mut q_shadow: Query<(&mut Visibility, &mut Transform, &ChildOf), With<HighLungeShadowPhase2>>,
    q_parent: Query<(&SmashTimers, &movements::PartialVelocity)>,
    time: Res<Time>,
) {
    for (mut visibility, mut transform, parent) in q_shadow.iter_mut() {
        let Ok((timers, partial_velocity)) = q_parent.get(parent.0) else {
            return;
        };
        if timers.timer.just_finished() {
            *visibility = Visibility::Hidden;
        }
        if timers.waiting.is_finished() {
            let displacement = -**partial_velocity * time.delta_secs();
            transform.translation.x += displacement.x;
            transform.translation.y += displacement.y;
        }
    }
}
