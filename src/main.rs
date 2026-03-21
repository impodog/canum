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
            avian2d::PhysicsPlugins::default().with_length_unit(32.0),
            canum_res::CanumResPlugin,
            canum_save::CanumSavePlugin,
            canum_play::CanumPlayPlugin,
        ))
        .add_systems(
            PreUpdate,
            |mut commands: Commands, mut flag: Local<bool>| {
                if !*flag {
                    let entity = commands
                        .spawn((
                            canum_play::player::Player,
                            canum_res::Animation::new("Cyan", Vec2::new(20.0, 20.0)),
                            canum_play::health::IntegerHealth::default(),
                        ))
                        .id();
                    commands.insert_resource(canum_play::player::PrimaryPlayer(entity));
                    *flag = true;
                    commands.trigger(canum_play::setup::StartSession {});
                }
            },
        )
        .run();
}
