use super::*;

pub(super) struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component, Default)]
#[require(Transform, Visibility)]
pub struct RulerLaser;

fn ruler_laser_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    const LASER_SIZE: Vec2 = vec2(10.0, 10.0);
    const COLLIDER_SIZE: Vec2 = vec2(10.0, 3.2);
}
