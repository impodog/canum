use super::*;

pub(super) struct SimplePlugin;

impl Plugin for SimplePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(fail3);
    }
}

fn fail3(_event: On<player::failure::PlayerFail>, mut save: ResMut<Save>, mut commands: Commands) {
    let value = save.progress.achievements.progress("FailCount");
    *value += 1;
    if *value == 3 {
        commands.trigger(GetAchievement::new("Fail3"));
    }
}
