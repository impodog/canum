use crate::prelude::*;

mod behaviors;
mod entry;

pub(super) struct WcatPlugin;

static WCAT_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Wcat".to_owned()));

impl Plugin for WcatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((entry::EntryPlugin, behaviors::BehaviorsPlugin));
        app.add_systems(OnEnter(WCAT_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(spawn_wcat_health_bar)));
        });
        app.add_systems(
            FixedPostUpdate,
            update_wcat_health_bar.run_if(in_state(WCAT_STATE.clone())),
        );
    }
}

#[derive(Component, Default)]
#[require(
    Animation::new("Wcat_Static", Vec2::new(100.0, 100.0)),
    Transform::from_translation(Vec3::new(200.0, 0.0, 14.37)),
    RigidBody::Dynamic,
    Collider::rectangle(44.0, 25.0),
    Mass(4.0),
    LockedAxes::ROTATION_LOCKED,
    Restitution::new(0.6),
    player::MoveMeBack,
    health::Friendly(false),
    health::ContactDamage { value: 120, projectile: false, order: consts::order::ENEMY_BOSS },
    movements::SpeedDecay(0.5),
    movements::AutoFlip::FLIP_LEFT,
    enemy::health::EnemyHealth::new(5000),
    enemy::health::DamageSound::new("Wcat_Damage"),
    player::victory::DefeatToWin::default(),
    StaggerTimes,
)]
pub struct WcatBoss;

/// The cat will get smarted if staggered too many times.
#[derive(Component, Default, Debug)]
pub struct StaggerTimes(pub i32);

fn spawn_wcat_health_bar(
    _event: On<entry::WcatFightStart>,
    save: Res<Save>,
    mut commands: Commands,
    q_bottom_center: Query<Entity, With<canum_ui::BottomCenter>>,
) {
    let Ok(bottom_center) = q_bottom_center.single() else {
        return;
    };
    if save.progress.selected_effects.contains("ShowHealth") {
        commands.spawn((
            ChildOf(bottom_center),
            canum_ui::bar::health_bar(
                Color::Srgba(Srgba::hex("#0eff52").unwrap()),
                Color::Srgba(Srgba::hex("#ffa72b").unwrap()),
                canum_ui::bar::HealthBar {
                    total: 5000.0,
                    current: 5000.0,
                },
            ),
        ));
    }
}

fn update_wcat_health_bar(
    q_health: Query<&enemy::health::EnemyHealth>,
    mut q_bar: Query<&mut canum_ui::bar::HealthBar>,
) {
    let Ok(health) = q_health.single() else {
        return;
    };
    let Ok(mut bar) = q_bar.single_mut() else {
        return;
    };
    bar.current = health.value as f32;
}
