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
                .with_length_unit(16.0)
                .with_collision_hooks::<canum_play::health::PhysicsHooks>(),
            canum_res::CanumResPlugin,
            canum_save::CanumSavePlugin,
            canum_play::CanumPlayPlugin,
            canum_ui::CanumUiPlugin,
            canum_fx::CanumFxPlugin,
        ))
        .add_plugins((canum_s1::CanumS1Plugin,))
        .insert_resource(Time::<Fixed>::from_hz(
            canum_res::config::CONFIG.client.update_freq as f64,
        ))
        .insert_resource(avian2d::prelude::Gravity::ZERO)
        .add_systems(
            PreUpdate,
            |mut commands: Commands, mut flag: Local<bool>| {
                if !*flag {
                    *flag = true;
                    commands.trigger(canum_play::setup::StartSession {
                        fight: "LobbySelect".to_owned(),
                    });
                }
            },
        )
        .run();
}
