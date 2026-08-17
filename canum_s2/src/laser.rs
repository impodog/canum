mod behaviors;
mod defeat;
mod entry;

use crate::*;

static LASER_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Laser".to_owned()));

pub(super) struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            behaviors::BehaviorsPlugin,
            entry::EntryPlugin,
            defeat::DefeatPlugin,
        ));
        app.add_systems(OnEnter(LASER_STATE.clone()), spawn_laser_health_bar);
        app.add_systems(
            FixedPostUpdate,
            update_laser_health_bar.run_if(in_state(LASER_STATE.clone())),
        );
    }
}

#[derive(Component, Default)]
#[require(
    Animation::new("Laser_Static", Vec2::new(64.0, 64.0)),
    Transform::from_translation(Vec3::new(0.0, 0.0, 14.37)),
    RigidBody::Dynamic,
    Collider::rectangle(50.0, 20.0),
    Mass(6.0),
    LockedAxes::TRANSLATION_LOCKED,
    Restitution::new(0.2),
    health::Friendly(false),
    health::ContactDamage { value: 120, projectile: false, order: consts::order::ENEMY_BOSS },
    movements::SpeedDecay(0.75),
    movements::AngularSpeedDecay,
    enemy::health::EnemyHealth::new(4000),
    enemy::health::DamageSound::new("Laser_Damage"),
    player::victory::DefeatToWin::default(),
    defeat::LaserDefeat
)]
pub struct LaserBoss;

fn spawn_laser_health_bar(
    save: Res<Save>,
    mut commands: Commands,
    q_bottom_center: Query<Entity, With<canum_ui::BottomCenter>>,
    mut title: ResMut<canum_res::window::WindowTitle>,
    lang: Res<Lang>,
) {
    let Ok(bottom_center) = q_bottom_center.single() else {
        return;
    };
    title.0 = lang.get("Laser_WindowTitle").to_owned();
    if save.progress.selected_effects.contains("ShowHealth") {
        commands.spawn((
            ChildOf(bottom_center),
            canum_ui::bar::health_bar(
                Color::Srgba(Srgba::hex("#ffb9db").unwrap()),
                Color::Srgba(Srgba::hex("#cc0000").unwrap()),
                canum_ui::bar::HealthBar {
                    total: 4000.0,
                    current: 4000.0,
                    ..default()
                },
            ),
        ));
    }
}

fn update_laser_health_bar(
    q_health: Query<&enemy::health::EnemyHealth, With<LaserBoss>>,
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
