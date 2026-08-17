use super::*;

pub(super) struct DefeatPlugin;

impl Plugin for DefeatPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<BreadDefeat>()
            .on_insert(|mut world, HookContext { entity, .. }| {
                world.commands().entity(entity).observe(on_defeat);
            });
    }
}

#[derive(Component, Default)]
pub struct BreadDefeat;

#[derive(Component)]
#[require(
    Animation::new("Bread_Meteor", vec2(64.0, 64.0)),
    RigidBody::Kinematic,
    movements::ForcedVelocity
)]
struct Meteor {
    bread: Entity,
}

fn on_defeat(
    event: On<enemy::health::EnemyDefeated>,
    mut commands: Commands,
    q_transform: Query<&GlobalTransform>,
) {
    commands
        .entity(event.entity)
        .remove::<Collider>()
        .remove::<RigidBody>();

    let Ok(transform) = q_transform.get(event.entity) else {
        return;
    };
    let position = transform.translation().xy();
    let displace = CONFIG.display.screen_size * position.signum();
    let start_position = position - displace;
    let meteor = commands
        .spawn((
            Meteor {
                bread: event.entity,
            },
            Transform::from_translation(vec3(start_position.x, start_position.y, 15.0)),
        ))
        .observe(on_displace_complete)
        .id();
    commands.spawn((
        ChildOf(meteor),
        enemy::movements::Displacement {
            curve: |x| x,
            displace,
            duration: Duration::from_secs(2),
            notify: Some(meteor),
        },
    ));
}

#[derive(Event)]
struct DefeatTimer(Entity);
canum_fx::wait_then_trigger!(DefeatTimerTrigger, DefeatTimer, Entity, 3.0);

fn on_displace_complete(
    event: On<enemy::movements::DisplacementComplete>,
    mut q_animation: Query<&mut Animation>,
    mut commands: Commands,
    q_meteor: Query<&Meteor>,
    q_music: Query<Entity, With<Music>>,
) {
    let Ok(meteor) = q_meteor.get(event.entity) else {
        return;
    };
    commands.entity(event.entity).despawn();
    commands.spawn((SessionOnly, Observer::new(defeat_bread)));
    commands
        .spawn(DefeatTimerTrigger(meteor.bread))
        .observe(DefeatTimerTrigger::observer);
    let Ok(mut animation) = q_animation.get_mut(meteor.bread) else {
        return;
    };
    animation.replace("Bread_BurntLoaf", false, None);

    for music in q_music.iter() {
        commands.entity(music).despawn();
    }
    commands.spawn(Sound::new("Bread_Explode").with_volume_add(10.0));
}

fn defeat_bread(event: On<DefeatTimer>, mut commands: Commands) {
    commands
        .entity(event.0)
        .remove::<player::victory::DefeatToWin>();
}
