mod background;
mod behaviors;
mod defeat;
mod entry;

use crate::prelude::*;

pub(super) struct AntPlugin;

static ANT_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Ant".to_owned()));

impl Plugin for AntPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            background::BackgroundPlugin,
            behaviors::BehaviorsPlugin,
            defeat::DefeatPlugin,
            entry::EntryPlugin,
        ));
        app.add_systems(OnEnter(ANT_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(spawn_ant)));
            commands.spawn((SessionOnly, Observer::new(spawn_ant_health_bar)));
        });
        app.world_mut()
            .register_component_hooks::<AntBoss>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .spawn((ChildOf(entity), entry::AntEntry(entity)));
            });
        app.add_systems(
            FixedPostUpdate,
            update_ant_health_bar.run_if(in_state(ANT_STATE.clone())),
        );
    }
}

#[derive(Component, Default)]
#[require(
    Animation::new("Ant_Run", Vec2::new(64.0, 64.0)),
    Transform::from_translation(Vec3::new(500.0, 0.0, 14.37)),
    RigidBody::Dynamic,
    Collider::rectangle(28.0, 22.0),
    Mass(1.0),
    LockedAxes::ROTATION_LOCKED,
    Restitution::new(0.2),
    health::Friendly(false),
    health::ContactDamage { value: 80, projectile: false, order: consts::order::ENEMY_BOSS },
    movements::SpeedDecay(0.6),
    movements::AutoFlip::FLIP_RIGHT,
    enemy::health::EnemyHealth::new(3500),
    enemy::health::DamageSound::new("Ant_Damage"),
    player::victory::DefeatToWin::default(),
    defeat::AntDefeat,
    // This will be removed later by `AntEntry`
    projectile::NoCollideBoundary,
)]
pub struct AntBoss;

fn spawn_ant(_event: On<background::AntSetupTimerComplete>, mut commands: Commands) {
    let ant = commands
        .spawn((AntBoss,))
        .observe(behaviors::change_ant_stage)
        .id();
    commands.spawn((ChildOf(ant), behaviors::AntBehaviors));
    commands.spawn((
        ChildOf(ant),
        enemy::health::EnemySensor,
        Collider::rectangle(60.0, 32.0),
    ));
}

fn spawn_ant_health_bar(
    _event: On<entry::AntEntryComplete>,
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
                Color::Srgba(Srgba::hex("#977d70").unwrap()),
                Color::Srgba(Srgba::hex("#101c2a").unwrap()),
                canum_ui::bar::HealthBar {
                    total: 3500.0,
                    current: 3500.0,
                    ..default()
                },
            ),
        ));
    }
}

fn update_ant_health_bar(
    q_health: Query<&enemy::health::EnemyHealth>,
    mut q_bar: Query<&mut canum_ui::bar::HealthBar>,
) {
    let Ok(health) = q_health.single() else {
        return;
    };
    let Ok(mut bar) = q_bar.single_mut() else {
        return;
    };
    bar.total = bar.total.max(health.value as f32);
    bar.current = health.value as f32;
}
