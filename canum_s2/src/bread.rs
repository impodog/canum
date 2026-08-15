mod behaviors;
mod entry;
mod projectiles;

use crate::*;

pub static BREAD_STATE: LazyLock<setup::Fight> = LazyLock::new(|| setup::Fight("Bread".to_owned()));

pub(super) struct BreadPlugin;

impl Plugin for BreadPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            entry::EntryPlugin,
            behaviors::BehaviorsPlugin,
            projectiles::ProjectilesPlugin,
        ));
        app.world_mut()
            .register_component_hooks::<BreadBoss>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().spawn((
                    ChildOf(entity),
                    enemy::health::EnemySensor,
                    Collider::circle(37.5),
                ));
            });
    }
}

#[derive(Component)]
#[require(
    Animation::new("Bread_Loaf", Vec2::new(80.0, 80.0)),
    Transform::from_translation(Vec3::new(0.0, 0.0, 14.37)),
    RigidBody::Dynamic,
    Collider::rectangle(70.0, 70.0),
    CollidingEntities,
    Mass(10.0),
    LockedAxes::ROTATION_LOCKED,
    Restitution::new(0.5),
    health::Friendly(false),
    health::ContactDamage { value: 130, projectile: false, order: consts::order::ENEMY_BOSS },
    movements::SpeedDecay(0.75),
    enemy::health::EnemyHealth::new(4400),
    player::victory::DefeatToWin::default(),
)]
pub struct BreadBoss {
    pub stage: u8,
}

impl Default for BreadBoss {
    fn default() -> Self {
        Self { stage: 1 }
    }
}

#[derive(EntityEvent, Deref)]
pub struct BreadNextStage {
    pub entity: Entity,
    #[deref]
    pub stage: u8,
}
