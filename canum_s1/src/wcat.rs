use crate::prelude::*;

mod behaviors;
mod entry;

pub(super) struct WcatPlugin;

static WCAT_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Wcat".to_owned()));

impl Plugin for WcatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((entry::EntryPlugin, behaviors::BehaviorsPlugin));
    }
}

#[derive(Component, Default)]
#[require(
    Animation::new("Wcat_Static", Vec2::new(100.0, 100.0)),
    Transform::from_translation(Vec3::new(200.0, 0.0, 14.37)),
    RigidBody::Dynamic,
    Collider::rectangle(50.0, 30.0),
    Mass(4.0),
    LockedAxes::ROTATION_LOCKED,
    Restitution::new(0.6),
    health::Friendly(false),
    health::ContactDamage { value: 120, projectile: false, order: consts::order::ENEMY_BOSS },
    movements::SpeedDecay(0.5),
    enemy::health::EnemyHealth::new(5000),
    enemy::health::DamageSound::new("Wcat_Damage"),
    player::victory::DefeatToWin::default(),
)]
pub struct WcatBoss;
