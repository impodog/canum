mod behaviors;
mod defeat;
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
            defeat::DefeatPlugin,
        ));
        app.world_mut()
            .register_component_hooks::<BreadBoss>()
            .on_add(|mut world, HookContext { entity, .. }| {
                world.commands().spawn((
                    ChildOf(entity),
                    enemy::health::EnemySensor,
                    Collider::rectangle(75.0, 75.0),
                ));
            });

        app.add_systems(OnEnter(BREAD_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(spawn_bread_health_bar)));
        });
    }
}

const BREAD_LENGTH: f32 = 80.0;
const BREAD_HALF_LENGTH: f32 = 40.0;

#[derive(Component)]
#[require(
    Animation::new("Bread_Loaf", Vec2::new(BREAD_LENGTH, BREAD_LENGTH)),
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
    enemy::health::DamageSound::new("Wcat_Damage"),
    player::victory::DefeatToWin::default(),
    defeat::BreadDefeat,
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

fn spawn_bread_health_bar(
    _event: On<entry::BreadStart>,
    save: Res<Save>,
    mut commands: Commands,
    bottom_center: Single<Entity, With<canum_ui::BottomCenter>>,
    bread: Single<Entity, With<BreadBoss>>,
) {
    if save.progress.selected_effects.contains("ShowHealth") {
        commands.spawn((
            ChildOf(bottom_center.entity()),
            canum_ui::bar::AssociatedBoss(bread.entity()),
            canum_ui::bar::health_bar(
                Color::Srgba(Srgba::hex("#efe1b1").unwrap()),
                Color::Srgba(Srgba::hex("#4a2d06").unwrap()),
                canum_ui::bar::HealthBar {
                    total: 4400.0,
                    current: 4400.0,
                    ..default()
                },
            ),
        ));
    }
}
