use crate::prelude::*;

mod background;
mod behaviors;
mod defeat;

pub(super) struct ApplePlugin;

static APPLE_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Apple".to_owned()));

impl Plugin for ApplePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            background::BackgroundPlugin,
            behaviors::BehaviorsPlugin,
            defeat::DefeatPlugin,
        ));
        app.add_observer(spawn_apple);
        app.world_mut()
            .register_component_hooks::<AppleBoss>()
            .on_add(|mut world, HookContext { entity, .. }| {
                let mut commands = world.commands();
                commands.spawn((ChildOf(entity), behaviors::AppleBehaviors));
                commands.entity(entity).observe(behaviors::change_stage);
            });
    }
}

/// Main marker for the apple boss.
#[derive(Component, Default)]
#[require(
    Animation::new("Apple_Static", Vec2::new(64.0, 64.0)),
    Transform::from_translation(Vec3::new(-10.0, 150.0, 14.37)),
    RigidBody::Dynamic,
    Collider::circle(20.0),
    Mass(3.0),
    LockedAxes::ROTATION_LOCKED,
    Restitution::new(0.5),
    health::Friendly(false),
    health::ContactDamage { value: 100, projectile: false, order: consts::order::ENEMY_BOSS },
    movements::SpeedDecay(0.5),
    enemy::health::EnemyHealth::new(3000),
    enemy::health::DamageSound::new("Apple_Damage"),
    player::victory::DefeatToWin::default(),
    defeat::AppleDefeat,
)]
pub struct AppleBoss;

fn spawn_apple(
    _event: On<background::AppleTreeBackgroundChanged>,
    mut commands: Commands,
    save: Res<Save>,
    q_bottom_center: Query<Entity, With<canum_ui::BottomCenter>>,
) {
    let apple = commands.spawn((AppleBoss,)).id();
    commands.spawn((
        canum_res::sound::Music,
        canum_res::sound::Sound::new("Apple_Bgm"),
    ));
    commands.spawn((
        ChildOf(apple),
        enemy::health::EnemySensor,
        Collider::rectangle(50.0, 50.0),
    ));

    let Ok(bottom_center) = q_bottom_center.single() else {
        return;
    };
    if save.progress.selected_effects.contains("ShowHealth") {
        commands.spawn((
            ChildOf(bottom_center),
            canum_ui::bar::AssociatedBoss(apple),
            canum_ui::bar::health_bar(
                Color::Srgba(Srgba::hex("#c73322").unwrap()),
                Color::Srgba(Srgba::hex("#c1c09f").unwrap()),
                canum_ui::bar::HealthBar {
                    total: 3000.0,
                    current: 3000.0,
                    ..default()
                },
            ),
        ));
    }
}
