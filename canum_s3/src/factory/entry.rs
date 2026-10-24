use std::collections::BTreeSet;

use super::*;

pub(super) struct EntryPlugin;

impl Plugin for EntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(FACTORY_STATE.clone()), setup_factory);
        app.add_systems(
            FixedUpdate,
            (scale_factory_box, move_player_and_factory_box_back).in_set(FactorySystems),
        );
    }
}

#[derive(Event, Default)]
pub struct FactoryInit;

#[derive(Event, Default)]
pub struct FactoryStart;

#[derive(Component, Default)]
#[require(SessionOnly, Collider, RigidBody::Dynamic)]
pub struct FactoryBox {
    pub start_scaling: bool,
}

const FACTORY_BOX_INITIAL_SCALE: f32 = 15.0;
const FACTORY_BOX_TARGET_SCALE: f32 = 2.0;
const FACTORY_BOX_SIZE: Vec2 = vec2(200.0, 100.0);

fn setup_factory(
    mut commands: Commands,
    mut window_title: ResMut<canum_res::window::WindowTitle>,
    lang: Res<Lang>,
) {
    window_title.0 = lang.get("Factory_WindowTitle").to_owned();

    canum_fx::session_observers!(commands, spawn_box, spawn_title);
    canum_fx::wait_then_trigger!(use commands, FactoryInit, 0.5);
    canum_fx::wait_then_trigger!(use commands, FactoryStart, 2.0);
}

fn spawn_box(
    _event: On<FactoryInit>,
    q_player: Query<Entity, With<player::Player>>,
    mut commands: Commands,
) {
    const SIZE: Vec2 = vec2(100.0, 50.0);
    const HALF_SIZE: Vec2 = vec2(SIZE.x * 0.5, SIZE.y * 0.5);
    const WIDTH: f32 = 3.0;

    let players = q_player.iter().collect::<BTreeSet<_>>();
    let collider = Collider::compound(vec![
        (
            vec2(-HALF_SIZE.x, 0.0),
            0.0,
            Collider::rectangle(WIDTH, SIZE.y),
        ),
        (
            vec2(HALF_SIZE.x, 0.0),
            0.0,
            Collider::rectangle(WIDTH, SIZE.y),
        ),
        (
            vec2(0.0, -HALF_SIZE.y),
            0.0,
            Collider::rectangle(SIZE.x, WIDTH),
        ),
        (
            vec2(0.0, HALF_SIZE.y),
            0.0,
            Collider::rectangle(SIZE.x, WIDTH),
        ),
    ]);

    commands.spawn((
        FactoryBox {
            start_scaling: true,
        },
        SpriteSheet::new("Factory_Box"),
        Transform::from_scale(vec3(
            FACTORY_BOX_INITIAL_SCALE,
            FACTORY_BOX_INITIAL_SCALE,
            0.0,
        ))
        .with_translation(vec3(0.0, 0.0, -5.5)),
        RigidBody::Dynamic,
        collider,
        Mass(3.0),
        AngularInertia(1.0),
        LockedAxes::ROTATION_LOCKED,
        Friction::new(0.3),
        Restitution::new(0.5),
        health::Friendly::FRIENDLY,
        health::AlwaysCollide(players),
        movements::SpeedDecay(0.008),
    ));
}

fn scale_factory_box(
    mut fbox: Single<(&mut FactoryBox, &mut Transform, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    if fbox.0.start_scaling {
        let new_value = (fbox.1.scale.x
            - time.delta_secs() * (FACTORY_BOX_INITIAL_SCALE - FACTORY_BOX_TARGET_SCALE))
            .max(FACTORY_BOX_TARGET_SCALE);
        fbox.1.scale.x = new_value;
        fbox.1.scale.y = new_value;
        if new_value == FACTORY_BOX_TARGET_SCALE {
            fbox.0.start_scaling = false;
            commands
                .entity(fbox.2)
                .remove::<projectile::NoCollideBoundary>();
        }
    }
}

fn spawn_title(
    _event: On<FactoryInit>,
    lang: Res<Lang>,
    fonts: Res<canum_ui::Fonts>,
    bottom_left: Single<Entity, With<canum_ui::BottomLeft>>,
    mut commands: Commands,
) {
    commands.spawn((
        ChildOf(bottom_left.entity()),
        canum_ui::text::popup_title(
            fonts.title.clone(),
            lang.get("Factory_BossTitle"),
            Duration::from_secs_f32(1.4),
        ),
    ));
}

fn keep_in(transform: &mut Transform, diff: Vec2, size: Vec2) {
    let size = size * 0.5;
    if diff.x > size.x {
        transform.translation.x -= 25.0;
    }
    if diff.x < -size.x {
        transform.translation.x += 25.0;
    }
    if diff.y > size.y {
        transform.translation.y -= 25.0;
    }
    if diff.y < -size.y {
        transform.translation.y += 25.0;
    }
}

#[allow(clippy::type_complexity)]
fn move_player_and_factory_box_back(
    mut q_player: Query<(&mut Transform, &GlobalTransform), With<player::Player>>,
    mut fbox: Single<
        (&mut Transform, &GlobalTransform),
        (With<FactoryBox>, Without<player::Player>),
    >,
) {
    let box_position = fbox.1.translation().xy();
    keep_in(&mut fbox.0, box_position, CONFIG.display.screen_size);
    for (mut transform, global_transform) in q_player.iter_mut() {
        let position = global_transform.translation().xy();
        keep_in(&mut transform, position - box_position, FACTORY_BOX_SIZE);
    }
}
