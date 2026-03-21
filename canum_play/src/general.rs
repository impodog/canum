use crate::prelude::*;

pub(super) struct GeneralPlugin;
impl Plugin for GeneralPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, do_speed_shrink);
    }
}

/// The entity shrinks their animation and hitbox and hitbox by a base number when their speed increases.
///
/// For each time the velocity of given number, the size is reduced by half.
#[derive(Component, Debug, Clone)]
#[require(Animation)]
pub struct SpeedShrink(pub f32);

fn do_speed_shrink(
    mut q_entity: Query<(&SpeedShrink, &mut Collider, &LinearVelocity, &mut Animation)>,
) {
    q_entity
        .par_iter_mut()
        .for_each(|(shrink, mut collider, linear_velocity, mut animation)| {
            let factor = (0.5f32).powf(linear_velocity.length() / shrink.0);
            collider.set_scale(Vec2::new(factor, 1.0), 6);
            animation.scale = Vec2::new(factor, 1.0);
        });
}
