use super::*;

pub(super) struct RunningPlugin;

impl Plugin for RunningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(RUNWAY_STATE.clone()), setup_runway);
        app.add_systems(
            FixedUpdate,
            (roll_objects, add_roll_to_objects)
                .chain()
                .run_if(in_state(RUNWAY_STATE.clone())),
        );
        app.add_systems(
            FixedPostUpdate,
            rotate_background.run_if(in_state(RUNWAY_STATE.clone())),
        );
    }
}

/// Makes all `PartialVelocity` marks with `RollingVelocity` move back, to simulate a running course.
#[derive(Resource, Default, Debug)]
pub struct RollingSpeed(pub f32);

#[derive(Component, Default)]
#[require(movements::PartialVelocity::unlinked(), movements::SpeedShrinkExclude)]
pub struct RollingVelocity;

fn roll_objects(
    speed: Res<RollingSpeed>,
    mut q_velocity: Query<&mut movements::PartialVelocity, With<RollingVelocity>>,
) {
    if speed.is_changed() {
        q_velocity.par_iter_mut().for_each(|mut velocity| {
            **velocity = vec2(0.0, speed.0);
        });
    }
}
fn add_roll_to_objects(
    speed: Res<RollingSpeed>,
    commands: ParallelCommands,
    q_forced_velocity: Query<Entity, Added<movements::ForcedVelocity>>,
) {
    q_forced_velocity.par_iter().for_each(|entity| {
        commands.command_scope(|mut commands| {
            commands.spawn((
                ChildOf(entity),
                movements::PartialVelocity {
                    linked: None,
                    velocity: vec2(0.0, speed.0),
                },
                RollingVelocity,
            ));
        });
    });
}

/// Marks one of the rotating backgrounds. Once it goes out of the screen it auto  the other one.
#[derive(Component, Default)]
#[require(SessionOnly, movements::ForcedVelocity, movements::DirectVelocity)]
struct BackgroundRotation;

fn setup_runway(mut commands: Commands) {
    let animation = Animation::new(
        "Runway_Background",
        vec2(
            CONFIG.display.screen_size.x,
            CONFIG.display.screen_size.y * 2.0,
        ),
    )
    .with_color(Color::linear_rgba(1.0, 1.0, 1.0, 0.7));
    commands.spawn((
        Transform::from_translation(vec3(0.0, CONFIG.display.screen_size.y * -0.5, 0.0)),
        animation.clone(),
        BackgroundRotation,
    ));
    commands.spawn((
        Transform::from_translation(vec3(0.0, CONFIG.display.screen_size.y * -2.5, 0.0)),
        animation,
        BackgroundRotation,
    ));
    commands.insert_resource(RollingSpeed(0.0));
}

fn rotate_background(mut q_background: Query<&mut Transform, With<BackgroundRotation>>) {
    static BACKGROUND_LIMIT: LazyLock<f32> = LazyLock::new(|| CONFIG.display.screen_size.y * 1.5);
    for mut transform in q_background.iter_mut() {
        if transform.translation.y >= *BACKGROUND_LIMIT {
            transform.translation.y -= CONFIG.display.screen_size.y * 4.0;
        }
    }
}
