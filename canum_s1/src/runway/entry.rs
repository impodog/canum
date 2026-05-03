use super::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(RUNWAY_STATE.clone()), init_timer);
        app.add_systems(
            FixedFirst,
            timer_tick.run_if(in_state(RUNWAY_STATE.clone())),
        );
    }
}

#[derive(Resource, Deref, DerefMut)]
struct EntryTimer(Timer);
impl Default for EntryTimer {
    fn default() -> Self {
        EntryTimer(Timer::from_seconds(0.2, TimerMode::Once))
    }
}

#[derive(Event, Default)]
pub struct StartRunwaySetup;

#[derive(Event, Default)]
pub struct RunwayOfficialStart;

fn init_timer(mut commands: Commands) {
    commands.insert_resource(EntryTimer::default());
    commands.spawn((SessionOnly, Observer::new(setup_runway)));
}
fn timer_tick(mut commands: Commands, mut timer: ResMut<EntryTimer>, time: Res<Time>) {
    if !timer.is_finished() {
        timer.tick(time.delta());
        if timer.just_finished() {
            commands.trigger(StartRunwaySetup);
        }
    }
}

fn setup_runway(
    _event: On<StartRunwaySetup>,
    mut q_player: Query<&mut Transform, With<player::Player>>,
    mut commands: Commands,
) {
    for mut transform in q_player.iter_mut() {
        transform.translation.x += CONFIG.display.screen_size.x * 0.25;
    }
    commands.spawn((
        SessionOnly,
        Collider::rectangle(3.0, CONFIG.display.screen_size.y),
        RigidBody::Static,
    ));
    commands.spawn((
        RunwayLion,
        Animation::new("Runway_Lion_Running", Vec2::new(64.0, 64.0)),
        Transform::from_translation(Vec3::new(
            -CONFIG.display.screen_size.x * 0.25,
            CONFIG.display.screen_size.y * 0.5,
            14.37,
        )),
        projectile::NoCollideBoundary,
    ));
}
