use super::*;

pub(super) struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<RulerLaser>()
            .on_add(ruler_laser_hook);
    }
}

#[derive(Component, Default, Debug, Clone, Copy)]
#[require(Transform, Visibility)]
pub struct RulerLaser {
    /// This doesn't have to be unit vector.
    pub direction: Vec2,
    pub double: bool,
}

fn ruler_laser_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    use canum_tool::weapon::laser::*;

    const LASER_SIZE: Vec2 = vec2(10.0, 10.0);
    const COLLIDER_SIZE: Vec2 = vec2(10.0, 3.2);

    let laser = *world
        .get::<RulerLaser>(entity)
        .expect("RulerLaser hook should get a entity with RulerLaser");
    let angle = laser.direction.normalize_or(vec2(1.0, 0.0)).to_angle();
    let laser_like = LaserLike {
        base_direction: Dir2::from_xy_unchecked(1.0, 0.0),
        middle: Animation::new("Ruler_Laser_Middle", LASER_SIZE),
        terminal: Animation::new("Ruler_Laser_Terminal", LASER_SIZE),
        length: COLLIDER_SIZE.x,
        collide_width: COLLIDER_SIZE.y,
        ignore_layer: LaserLayer::LASER_ENEMY,
    };

    if laser.double {
        world.commands().spawn((
            laser_like.clone(),
            ChildOf(entity),
            Transform::from_translation(vec3(0.0, 0.0, rand_offset()))
                .with_rotation(Quat::from_rotation_z(-angle)),
            health::ContactDamage {
                value: 50,
                projectile: false,
                order: consts::order::ENEMY_PROJ,
            },
            health::Friendly::UNFRIENDLY,
        ));
    }
    world.commands().spawn((
        laser_like,
        ChildOf(entity),
        Transform::from_translation(vec3(0.0, 0.0, rand_offset()))
            .with_rotation(Quat::from_rotation_z(angle)),
        health::ContactDamage {
            value: 50,
            projectile: false,
            order: consts::order::ENEMY_PROJ,
        },
        health::Friendly::UNFRIENDLY,
    ));
}
