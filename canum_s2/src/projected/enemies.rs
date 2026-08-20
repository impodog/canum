mod mantis;

use super::*;
use enemy::behavior::*;
use std::collections::{HashMap, VecDeque};

pub(super) struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((mantis::MantisPlugin,));
        app.world_mut()
            .register_component_hooks::<ProjectedEnemy>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(enemy_defeated);
            });

        app.add_systems(
            OnEnter(PROJECTED_STATE.clone()),
            |mut commands: Commands| {
                commands.spawn((SessionOnly, Observer::new(init_enemy_waves)));
            },
        );
        app.add_systems(
            FixedUpdate,
            (update_health_bar, spawn_enemy_wave).in_set(ProjectedSet),
        );
    }
}

#[derive(Component, Default)]
#[require(
    SessionOnly,
    Animation,
    RigidBody::Dynamic,
    Collider,
    Mass(3.0),
    LockedAxes::ROTATION_LOCKED,
    Restitution::new(0.2),
    health::Friendly(false),
    health::ContactDamage { value: 120, projectile: false, order: consts::order::ENEMY_MINION },
    movements::SpeedDecay(0.4),
    enemy::health::EnemyHealth,
    enemy::health::DamageSound::new("Projected_ShadowHit"),
    projectile::NoCollideBoundary,
)]
pub struct ProjectedEnemy;

fn enemy_defeated(
    event: On<health::Damage>,
    q_health: Query<(&enemy::health::EnemyHealth, &GlobalTransform, &Animation)>,
    mut commands: Commands,
) {
    let Ok((health, global_transform, animation)) = q_health.get(event.entity) else {
        return;
    };
    if health.value <= 0 {
        let position = global_transform.translation() + vec3(0.0, 0.0, -0.1);
        commands.entity(event.entity).try_despawn();
        commands.spawn((
            Animation::new("Projected_ShadowDefeated", animation.size).once_then_despawn(),
            Transform::from_translation(position),
        ));
        commands.spawn(Sound::new("Projected_ShadowDefeated"));
    }
}

static ENEMY_VALUES: LazyLock<HashMap<String, i32>> = LazyLock::new(|| {
    let iter = [("Mantis", 10)]
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value));
    HashMap::from_iter(iter)
});

#[derive(Debug, serde::Deserialize)]
pub struct Wave {
    pub enemies: Vec<(String, usize)>,
    pub timeout: f32,
}

/// Stores all waves of this level.
#[derive(Resource, Deref, DerefMut, Debug, serde::Deserialize)]
pub struct Waves {
    #[deref]
    pub waves: VecDeque<Wave>,
    pub current_value: i32,
    pub total_value: i32,
}
impl Waves {
    fn calc_values(&mut self) {
        self.total_value = 0;
        self.current_value = 0;
        for wave in self.waves.iter() {
            for (enemy, count) in wave.enemies.iter() {
                self.total_value += ENEMY_VALUES.get(enemy).copied().unwrap_or(0) * *count as i32;
            }
        }
    }
}
impl Default for Waves {
    fn default() -> Self {
        Self {
            waves: default(),
            current_value: 1,
            total_value: 1,
        }
    }
}

#[derive(Component, Default)]
struct ProjectedTimerBar;

fn init_enemy_waves(
    _event: On<entry::ProjectedStart>,
    mut commands: Commands,
    bottom_center: Single<Entity, With<canum_ui::BottomCenter>>,
) {
    commands.init_resource::<Waves>();
    let waves: Waves = if let Some(waves) = CONFIG.values.custom.get("Projected_Waves") {
        match waves.clone().into_rust::<VecDeque<Wave>>() {
            Ok(wave_enemies) => {
                let mut waves = Waves {
                    waves: wave_enemies,
                    ..default()
                };
                waves.calc_values();
                waves
            }
            Err(err) => {
                error!("Unable to parse projected enemy waves: {err}. The game will not continue!");
                default()
            }
        }
    } else {
        warn!("Unable to find enemy waves for projected. The game will not continue!");
        default()
    };
    commands.spawn((
        ChildOf(bottom_center.entity()),
        ProjectedTimerBar,
        canum_ui::bar::health_bar(
            Color::srgb_u8(150, 255, 20),
            Color::srgb_u8(30, 30, 30),
            canum_ui::bar::HealthBar {
                total: waves.total_value as f32,
                current: waves.current_value as f32,
                ..default()
            },
        ),
    ));
    commands.insert_resource(waves);
}

#[derive(Debug, Deref, DerefMut)]
struct SpawnTimeout(Timer);
impl Default for SpawnTimeout {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Once))
    }
}

fn spawn_enemy_wave(
    mut commands: Commands,
    q_enemy: Query<(), With<ProjectedEnemy>>,
    waves: Option<ResMut<Waves>>,
    mut timeout: Local<SpawnTimeout>,
    time: Res<Time>,
) {
    timeout.tick(time.delta());
    if q_enemy.iter().next().is_some() && !timeout.is_finished() {
        return;
    }
    let Some(mut waves) = waves else {
        return;
    };
    if let Some(wave) = waves.waves.pop_front() {
        timeout.0 = Timer::from_seconds(wave.timeout, TimerMode::Once);
        for (enemy, count) in wave.enemies.into_iter() {
            for _ in 0..count {
                let y = rand::random_range(
                    -CONFIG.display.half_virtual_size.1 + 32.0
                        ..CONFIG.display.half_virtual_size.1 - 32.0,
                );
                let x = (CONFIG.display.half_virtual_size.0 + 32.0) * rand_sign();
                let transform = Transform::from_translation(vec3(x, y, 5.0));
                match enemy.as_str() {
                    "Mantis" => {
                        commands.spawn((mantis::Mantis, transform));
                    }
                    _ => {
                        warn!("Unknown projected enemy: {enemy}");
                    }
                }
            }
            waves.current_value += ENEMY_VALUES.get(&enemy).copied().unwrap_or(0) * count as i32;
        }
    } else {
        timeout.0 = Timer::from_seconds(10000.0, TimerMode::Once);
    }
}

fn update_health_bar(
    waves: Option<Res<Waves>>,
    mut bar: Single<&mut canum_ui::bar::HealthBar, With<ProjectedTimerBar>>,
    time: Res<Time>,
) {
    const INCREMENT_SPEED: f32 = 10.0;
    if let Some(waves) = waves {
        let target = waves.current_value as f32;
        if bar.current < target {
            bar.current = (bar.current + time.delta_secs() * INCREMENT_SPEED).min(target);
        }
    }
}
