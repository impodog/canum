use crate::prelude::*;

pub(super) struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, update_integer_health);
    }
}

#[derive(Component, Debug)]
#[require(Node)]
struct IntegerHealthUi {
    /// This UI watches this health bar.
    pub watch: Entity,
    /// The number of health remaining in the UI. This will have a delay to the actual, which will be detected and animation will be played.
    pub ui_count: usize,
}

pub(crate) fn integer_health(watch: Entity, number: usize) -> impl Bundle {
    let mut sub_nodes = Vec::new();
    for _ in 0..number {
        sub_nodes.push((
            Node {
                align_self: AlignSelf::Center,
                width: px(75.0),
                height: px(75.0),
                ..default()
            },
            Animation::new("BasicHp", Vec2::new(32.0, 32.0)).with_pause(0),
        ));
    }
    (
        IntegerHealthUi {
            watch,
            ui_count: number,
        },
        Node {
            align_content: AlignContent::Start,
            justify_content: JustifyContent::Start,
            column_gap: px(10.0),
            ..default()
        },
        Children::spawn(sub_nodes),
    )
}

fn update_integer_health(
    mut q_integer_health: Query<(&mut IntegerHealthUi, &Children)>,
    mut q_animation: Query<&mut Animation>,
    q_health: Query<&canum_play::player::health::IntegerHealth>,
) {
    for (mut ui, children) in q_integer_health.iter_mut() {
        let Ok(health) = q_health.get(ui.watch) else {
            continue;
        };
        while ui.ui_count > health.count as usize {
            ui.ui_count -= 1;
            let Some(child) = children.get(ui.ui_count) else {
                break;
            };
            let Ok(mut animation) = q_animation.get_mut(*child) else {
                continue;
            };
            // Unleash the animation turning into cross.
            animation.pause = None;
        }
    }
}
