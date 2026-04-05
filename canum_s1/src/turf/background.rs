use crate::prelude::*;

pub(super) struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(super::TURF_STATE.clone()), setup_turf);
    }
}

fn setup_turf(
    mut commands: Commands,
    mut window_title: ResMut<canum_res::window::WindowTitle>,
    lang: Res<Lang>,
) {
    commands.spawn((
        SessionOnly,
        canum_res::background::Background::new(CONFIG.display.screen_size),
        Animation::new("Turf_Grassland", CONFIG.display.screen_size),
    ));
    window_title.0 = lang.get("Turf_WindowTitle").to_owned();
}
