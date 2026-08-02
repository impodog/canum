use super::*;

pub(super) struct BarPlugin;

impl Plugin for BarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(WCAT_STATE.clone()), |mut commands: Commands| {
            commands.spawn((SessionOnly, Observer::new(spawn_bar)));
        });
    }
}

#[derive(Component, Default)]
pub struct CanStaggerWcat;

#[derive(Component, Default)]
#[require(
    Animation::new("Wcat_Bar", vec2(160.0, 24.0)),
    Collider::rectangle(160.0, 24.0),
    RigidBody::Dynamic,
    Mass(3.5),
    CanStaggerWcat
)]
pub struct CentralBar;

#[derive(Component, Default)]
#[require(Collider::circle(1.0), RigidBody::Static)]
pub struct CentralHolder;

fn spawn_bar(_event: On<super::entry::WcatStart>, mut commands: Commands) {
    let holder = commands.spawn(CentralHolder).id();
    let bar = commands.spawn(CentralBar).id();
    commands.spawn(RevoluteJoint::new(holder, bar).with_anchor(vec2(0.0, 0.0)));
}
