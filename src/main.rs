use bevy::prelude::*;

fn main() {
    let run_path = std::path::Path::new(".").canonicalize().unwrap();
    for arg in std::env::args() {
        match arg.trim() {
            "--no-save" => {
                // Here uses println! because logger is not initialized yet.
                println!("Save data will not be written when running.");
                canum_save::NO_SAVE.get_or_init(|| true);
            }
            "--no-run" => {
                return;
            }
            _ => {}
        }
    }
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
            bevy_image_font::ImageFontPlugin,
        ))
        .add_plugins((
            canum_res::CanumResPlugin,
            canum_save::CanumSavePlugin,
            canum_play::CanumPlayPlugin,
            canum_ui::CanumUiPlugin,
            canum_fx::CanumFxPlugin,
            canum_addons::CanumAddonsPlugin,
            canum_tool::CanumToolPlugin,
        ))
        .add_plugins((canum_s1::CanumS1Plugin, canum_s2::CanumS2Plugin))
        .insert_resource(Time::<Fixed>::from_hz(
            canum_res::config::CONFIG.client.update_freq as f64,
        )).insert_resource(ClearColor(Color::BLACK))
        .insert_resource(avian2d::prelude::Gravity::ZERO)
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(BackgroundColor(Color::linear_rgb(0.05, 0.05, 0.05)));
        })
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
        .add_systems(
            Update,
            |mut commands: Commands,
             q_bottom: Query<Entity, With<canum_ui::BottomCenter>>,
             key: Res<ButtonInput<KeyCode>>| {
                if key.just_pressed(KeyCode::KeyQ) {
                    let Ok(bottom) = q_bottom.single() else {
                        return;
                    };
                    commands.spawn((
                        ChildOf(bottom),
                        canum_ui::dialogue::Dialogue {
                            sections: vec![canum_ui::dialogue::DialogueSection {
                                title: "Dialogue Text Title Here".to_owned(),
                                icon_image: "Empty".to_owned(),
                                text: vec!["If you see this, I probably, no, definitely forgot to remove this test code".to_owned()],
                            }],
                            background: "Ui_Dialogue_Background".to_owned(),
                            index: 0,
                        },
                    ));
                }
            },
        )
        .run();
}
