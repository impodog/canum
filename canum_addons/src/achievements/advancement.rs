use super::*;

pub(super) struct AdvancementPlugin;

impl Plugin for AdvancementPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(stage_complete);
    }
}

fn stage_complete(event: On<player::victory::CompletedTasks>, mut commands: Commands) {
    match event.fight.as_str() {
        "Wcat" => commands.trigger(GetAchievement::new("Stage1Complete")),
        "Ruler" => commands.trigger(GetAchievement::new("Stage2Complete")),
        _ => {}
    }
}
