use bevy::prelude::*;

fn main() {
    let run_path = std::path::Path::new(".").canonicalize().unwrap();
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
                    file_path: run_path.to_string_lossy().into_owned(),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            avian2d::PhysicsPlugins::default()
                .with_length_unit(32.0)
                .with_collision_hooks::<canum_play::health::FriendlyHooks>(),
            canum_res::CanumResPlugin,
            canum_save::CanumSavePlugin,
            canum_play::CanumPlayPlugin,
        ))
        .add_systems(PostStartup, |mut commands: Commands| {
            commands.insert_resource(avian2d::prelude::Gravity::ZERO);
        })
        .add_systems(
            PreUpdate,
            |mut commands: Commands, mut flag: Local<bool>| {
                if !*flag {
                    *flag = true;
                    commands.trigger(canum_play::setup::StartSession {});
                }
            },
        )
        .run();
}
