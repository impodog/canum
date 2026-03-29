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
                .with_collision_hooks::<canum_play::health::FriendlyHooks>(),
            canum_res::CanumResPlugin,
            canum_save::CanumSavePlugin,
            canum_play::CanumPlayPlugin,
            canum_ui::CanumUiPlugin,
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
                        fight: "Apple".to_owned(),
                    });
                }
            },
        )
        .add_systems(
            PreUpdate,
            |mut commands: Commands,
             primary_player: Option<Res<canum_play::player::PrimaryPlayer>>| {
                if let Some(primary_player) = primary_player
                    && rand::random_bool(0.005)
                {
                    commands.trigger(canum_play::health::Damage {
                        entity: primary_player.0,
                        value: 1,
                        order: 255,
                    });
                }
            },
        )
        .run();
}
