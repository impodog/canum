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
        });
        app.world_mut()
            .register_component_hooks::<AntBoss>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world
                    .commands()
                    .spawn((ChildOf(entity), entry::AntEntry(entity)));
            });
    }
}

#[derive(Component, Default)]
#[require(
    Animation::new("Ant_Run", Vec2::new(64.0, 64.0)),
    Transform::from_translation(Vec3::new(500.0, 0.0, 14.37)),
    RigidBody::Dynamic,
    Collider::rectangle(28.0, 20.0),
    Mass(1.0),
    LockedAxes::ROTATION_LOCKED,
    Restitution::new(0.2),
    health::Friendly(false),
    health::ContactDamage { value: 80, projectile: false, order: consts::order::ENEMY_BOSS },
    movements::SpeedDecay(0.6),
    movements::AutoFlip::FLIP_RIGHT,
    enemy::health::EnemyHealth::new(3000),
    enemy::health::DamageSound::new("Ant_Damage"),
    player::victory::DefeatToWin::default(),
    // This will be removed later by `AntEntry`
    projectile::NoCollideBoundary,
)]
pub struct AntBoss;

fn spawn_ant(_event: On<background::AntSetupTimerComplete>, mut commands: Commands) {
    commands.spawn((AntBoss,));
}
