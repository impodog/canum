use crate::prelude::*;

pub(super) struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPreUpdate, bump_away);
    }
}

/// Only allow entities with relative velocity in the same direction as this.
#[derive(Component, Debug, Clone, Copy)]
#[require(
    LinearVelocity,
    Collider,
    Transform,
    ActiveCollisionHooks::FILTER_PAIRS
)]
pub struct OnewayCollision(pub Dir2);
impl Default for OnewayCollision {
    fn default() -> Self {
        Self(Dir2::new_unchecked(Vec2::from_angle(0.0)))
    }
}
impl OnewayCollision {
    pub fn test_velocity(&self, velocity: Vec2) -> bool {
        velocity.dot(self.0.as_vec2()) > 0.0
    }
}

/// For screen border colliders, try to bump the player away when colliding.
#[derive(Component, Debug, Clone, Copy)]
#[require(Collider, CollidingEntities)]
pub struct BumpAway {
    pub direction: Dir2,
    pub strength: f32,
}

/// Cancels the `CollisionDisabled` marker caused by `BumpAway`.
#[derive(Component)]
struct BumpAwayCanceller(Entity);

fn bump_away(q_bump: Query<(&BumpAway, &CollidingEntities)>, commands: ParallelCommands) {
    const DURATION: Duration = Duration::from_millis(100);
    q_bump.par_iter().for_each(|(bump, colliding)| {
        for entity in colliding.iter() {
            let direction = bump
                .direction
                .rotate(Vec2::from_angle(rand::random_range(-0.2..0.2)));
            commands.command_scope(|mut commands| {
                let canceller = commands
                    .spawn(BumpAwayCanceller(*entity))
                    .observe(cancel_bump_away)
                    .id();
                commands.entity(*entity).insert(ColliderDisabled);
                commands.spawn((
                    ChildOf(*entity),
                    crate::enemy::movements::Displacement {
                        curve: |x| QuarticOutCurve.sample(x).unwrap(),
                        displace: direction * bump.strength,
                        duration: DURATION,
                        notify: Some(canceller),
                    },
                ));
            });
        }
    });
}

fn cancel_bump_away(
    event: On<enemy::movements::DisplacementComplete>,
    q_cancel: Query<&BumpAwayCanceller>,
    mut commands: Commands,
) {
    let Ok(cancel) = q_cancel.get(event.entity) else {
        return;
    };
    commands.entity(cancel.0).try_remove::<ColliderDisabled>();
}
