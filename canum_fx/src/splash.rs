use std::{collections::BTreeSet, sync::Mutex, time::Duration};

use crate::prelude::*;

pub(super) struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_splash);
        app.add_systems(
            FixedPostUpdate,
            (despawn_completed_splash, init_splash_drops).chain(),
        );
        app.add_systems(FixedUpdate, move_splash_drops);
        app.add_systems(
            FixedPostUpdate,
            (despawn_completed_gravity_splash, init_gravity_splash_drops).chain(),
        );
        app.add_systems(FixedUpdate, move_gravity_splash_drops);
    }
}

/// Creates a splash effect centered around this entity.
/// Don't add this a component, instead spawn it as a children.
#[derive(Component, Default, Debug)]
#[require(SplashMaterial, Transform, Visibility)]
pub struct Splash {
    pub color: Color,
    pub duration: Duration,
    pub number: usize,
}
#[derive(Component, Default)]
struct SplashMaterial {
    material: Handle<ColorMaterial>,
    children: BTreeSet<Entity>,
}

#[derive(Component)]
#[require(Transform, Visibility)]
struct SplashDrop {
    parent: Entity,
    start_time: Duration,
    direction: f32,
}

#[derive(Resource)]
struct SplashMeshes {
    sector: Handle<Mesh>,
    triangle: Handle<Mesh>,
}

fn setup_splash(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    const RADIUS: f32 = 4.0;
    const HEIGHT: f32 = 15.0;
    let sector = meshes.add(CircularSector::new(RADIUS, std::f32::consts::FRAC_PI_2));
    let triangle = meshes.add(Triangle2d::new(
        Vec2::new(-RADIUS, 0.0),
        Vec2::new(RADIUS, 0.0),
        Vec2::new(0.0, -HEIGHT),
    ));
    commands.insert_resource(SplashMeshes { sector, triangle });
}

fn init_splash_drops(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut q_splash: Query<(Entity, &mut Splash, &mut SplashMaterial, &GlobalTransform)>,
    meshes: Res<SplashMeshes>,
    time: Res<Time>,
) {
    for (entity, mut splash, mut material, global_transform) in q_splash.iter_mut() {
        if splash.number == 0 {
            continue;
        }
        if splash.is_added() {
            material.material = materials.add(ColorMaterial {
                color: splash.color,
                alpha_mode: bevy::sprite_render::AlphaMode2d::Blend,
                ..default()
            });
            // Prevents the first frame from adding splash drops, and wait for global transform to update.
            return;
        }
        if rand::random_bool(0.8) {
            splash.number = splash.number.saturating_sub(1);
            let direction = rand::random_range(0.0..(std::f32::consts::PI * 2.0));
            let mut transform = Transform::from_translation(global_transform.translation());
            transform.rotate_local_z(direction - std::f32::consts::FRAC_PI_2);
            let child = commands
                .spawn((
                    transform,
                    children![
                        (
                            Mesh2d(meshes.sector.clone()),
                            MeshMaterial2d(material.material.clone())
                        ),
                        (
                            Mesh2d(meshes.triangle.clone()),
                            MeshMaterial2d(material.material.clone())
                        )
                    ],
                    SplashDrop {
                        parent: entity,
                        start_time: time.elapsed(),
                        direction,
                    },
                ))
                .id();
            material.children.insert(child);
        }
    }
}

fn move_splash_drops(
    commands: ParallelCommands,
    mut q_splash_drop: Query<(Entity, &mut Transform, &SplashDrop)>,
    q_splash: Query<&Splash>,
    time: Res<Time>,
) {
    q_splash_drop
        .par_iter_mut()
        .for_each(|(entity, mut transform, splash_drop)| {
            let Ok(splash) = q_splash.get(splash_drop.parent) else {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                });
                return;
            };
            let elapsed = time.elapsed() - splash_drop.start_time;
            if elapsed >= splash.duration {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                });
                return;
            }
            let speed = (1.1 - elapsed.as_secs_f32() / splash.duration.as_secs_f32()) * 15.0;
            let (sin, cos) = splash_drop.direction.sin_cos();
            transform.translation += Vec3::new(cos * speed, sin * speed, 0.0);
        });
}

fn despawn_completed_splash(
    commands: ParallelCommands,
    mut q_splash: Query<(Entity, &Splash, &mut SplashMaterial)>,
    q_exists: Query<()>,
) {
    q_splash
        .par_iter_mut()
        .for_each(|(entity, splash, mut splash_material)| {
            let mut to_remove = Vec::new();
            for child in splash_material.children.iter().copied() {
                if q_exists.get(child).is_err() {
                    to_remove.push(child);
                }
            }
            for child in to_remove.into_iter() {
                splash_material.children.remove(&child);
            }
            if splash.number == 0 && splash_material.children.is_empty() {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                });
            }
        });
}

/// Creates a splash effect, with drops falling and fading out.
/// Don't add this a component, instead spawn it as a children.
#[derive(Component, Default, Debug)]
#[require(GravitySplashMaterial, Transform, Visibility)]
pub struct GravitySplash {
    pub color: Color,
    pub duration: Duration,
    pub start_speed: f32,
    pub number: usize,
}

const GRAVITY_DROP_FADING_LEVELS: usize = 10;
#[derive(Component, Default)]
struct GravitySplashMaterial {
    material: [Handle<ColorMaterial>; GRAVITY_DROP_FADING_LEVELS],
    children: BTreeSet<Entity>,
}

#[derive(Component)]
#[require(Transform, Visibility, LinearVelocity)]
struct GravitySplashDrop {
    parent: Entity,
    start_time: Duration,
    previous_index: usize,
}

fn init_gravity_splash_drops(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut q_splash: Query<(
        Entity,
        &mut GravitySplash,
        &mut GravitySplashMaterial,
        &GlobalTransform,
    )>,
    meshes: Res<SplashMeshes>,
    time: Res<Time>,
) {
    for (entity, mut splash, mut material, global_transform) in q_splash.iter_mut() {
        if splash.number == 0 {
            continue;
        }
        if splash.is_added() {
            let alpha = splash.color.alpha();
            for i in 0..GRAVITY_DROP_FADING_LEVELS {
                let mut color = splash.color;
                color.set_alpha(
                    alpha * (GRAVITY_DROP_FADING_LEVELS - i) as f32
                        / GRAVITY_DROP_FADING_LEVELS as f32,
                );
                material.material[i] = materials.add(ColorMaterial {
                    color,
                    alpha_mode: bevy::sprite_render::AlphaMode2d::Blend,
                    ..default()
                });
            }
            // Same as above.
            return;
        }
        let number = (crate::math::rand_normal(2.0, 0.5) as usize).clamp(0, splash.number);
        for _ in 0..number {
            splash.number = splash.number.saturating_sub(1);
            let direction = rand::random_range(0.5..(std::f32::consts::PI - 0.5));
            let mut transform = Transform::from_translation(global_transform.translation());
            transform.rotate_local_z(direction - std::f32::consts::FRAC_PI_2);
            let child = commands
                .spawn((
                    transform,
                    children![
                        (
                            Mesh2d(meshes.sector.clone()),
                            MeshMaterial2d(material.material[0].clone())
                        ),
                        (
                            Mesh2d(meshes.triangle.clone()),
                            MeshMaterial2d(material.material[0].clone())
                        )
                    ],
                    LinearVelocity(Vec2::from_angle(direction) * splash.start_speed),
                    GravitySplashDrop {
                        parent: entity,
                        start_time: time.elapsed(),
                        previous_index: 0,
                    },
                ))
                .id();
            material.children.insert(child);
        }
    }
}

fn move_gravity_splash_drops(
    commands: ParallelCommands,
    mut q_splash_drop: Query<(
        Entity,
        &mut Transform,
        &mut LinearVelocity,
        &mut GravitySplashDrop,
        &Children,
    )>,
    q_material: Query<&mut MeshMaterial2d<ColorMaterial>>,
    q_splash: Query<(&GravitySplash, &GravitySplashMaterial)>,
    time: Res<Time>,
) {
    const GRAVITY: Vec2 = vec2(0.0, -500.0);
    let q_material = Mutex::new(q_material);
    q_splash_drop.par_iter_mut().for_each(
        |(entity, mut transform, mut linear_velocity, mut splash_drop, children)| {
            let Ok((splash, splash_material)) = q_splash.get(splash_drop.parent) else {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                });
                return;
            };
            let elapsed = time.elapsed() - splash_drop.start_time;
            if elapsed >= splash.duration {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                });
                return;
            }
            let percentage = elapsed.as_secs_f32() / splash.duration.as_secs_f32()
                * GRAVITY_DROP_FADING_LEVELS as f32;
            let index = (percentage as usize).clamp(0, GRAVITY_DROP_FADING_LEVELS - 1);
            if index != splash_drop.previous_index {
                let mut q_material = q_material.lock().unwrap();
                for child in children.iter() {
                    let Ok(mut material) = q_material.get_mut(child) else {
                        continue;
                    };
                    **material = splash_material.material[index].clone();
                }
                splash_drop.previous_index = index;
            }
            transform.translation.x += linear_velocity.x * time.delta_secs();
            transform.translation.y += linear_velocity.y * time.delta_secs();
            **linear_velocity += GRAVITY * time.delta_secs();
            transform.rotation =
                Quat::from_rotation_z(linear_velocity.to_angle() - std::f32::consts::FRAC_PI_2);
        },
    );
}

fn despawn_completed_gravity_splash(
    commands: ParallelCommands,
    mut q_splash: Query<(Entity, &GravitySplash, &mut GravitySplashMaterial)>,
    q_exists: Query<()>,
) {
    q_splash
        .par_iter_mut()
        .for_each(|(entity, splash, mut splash_material)| {
            let mut to_remove = Vec::new();
            for child in splash_material.children.iter().copied() {
                if q_exists.get(child).is_err() {
                    to_remove.push(child);
                }
            }
            for child in to_remove.into_iter() {
                splash_material.children.remove(&child);
            }
            if splash.number == 0 && splash_material.children.is_empty() {
                commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                });
            }
        });
}
