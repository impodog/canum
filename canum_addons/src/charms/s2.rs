use super::*;

pub(super) struct S2Plugin;

impl Plugin for S2Plugin {
    fn build(&self, app: &mut App) {
        app.add_observer(add_central_play)
            .add_observer(spawn_central_play_ui);
        app.add_systems(FixedUpdate, central_play_work);
    }
}

#[derive(Component, Default)]
#[require(SessionOnly)]
pub struct CentralPlay {
    pub prev_addition: f64,
}
const MAX_ADDITION: f32 = 0.1;

fn add_central_play(event: On<setup::StartSessionAction>, save: Res<Save>, mut commands: Commands) {
    if save.progress.selected_effects.contains("CentralPlay") {
        commands
            .entity(event.player_entity)
            .insert(CentralPlay { prev_addition: 0.0 });
    }
}

fn central_play_work(
    mut central_play: Single<(&GlobalTransform, &mut CentralPlay)>,
    camera: Single<&GlobalTransform, With<canum_res::camera::PixelCamera>>,
    mut args: ResMut<crate::stats::player_stats::DynamicPlayerArguments>,
    mut ui: Single<&mut canum_ui::bar::HealthBar, With<CentralPlayUi>>,
) {
    let camera_position = camera.translation().xy();
    let position = central_play.0.translation().xy();
    let distance = position.distance(camera_position);
    let addition = if distance <= 50.0 {
        MAX_ADDITION
    } else {
        MAX_ADDITION * (625.0 / (distance + 200.0) - 1.5)
    } as f64;
    args.damage_addition = addition - central_play.1.prev_addition;
    central_play.1.prev_addition = addition;

    ui.current = addition as f32;
}

#[derive(Component, Default, Clone, Copy)]
#[require(Node)]
struct CentralPlayUi;

fn spawn_central_play_ui(
    _event: On<setup::StartSessionAddedUi>,
    q_central_play: Query<(), With<CentralPlay>>,
    left_top: Single<Entity, With<canum_ui::TopLeft>>,
    mut commands: Commands,
) {
    if q_central_play.iter().next().is_some() {
        commands.spawn((
            ChildOf(left_top.entity()),
            CentralPlayUi,
            canum_ui::bar::health_bar(
                Color::Srgba(Srgba::hex("#ff5506").unwrap()),
                Color::Srgba(Srgba::hex("#0b4303").unwrap()),
                canum_ui::bar::HealthBar {
                    current: 0.0,
                    total: MAX_ADDITION,
                    width: 50.0,
                    ..default()
                },
            ),
        ));
    }
}
