use crate::prelude::*;

pub(super) struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, update_integer_health);
        app.init_resource::<HealthDetails>();
        app.add_observer(spawn_health_uis);
    }
}

#[derive(Resource, Debug, Clone)]
pub enum HealthDetails {
    BasicHp(BasicHpDetails),
}
impl Default for HealthDetails {
    fn default() -> Self {
        Self::BasicHp(default())
    }
}

#[derive(Debug, Clone)]
pub struct BasicHpDetails {
    /// Image array of health pips. This should be of same length of `count`.
    pub array: Vec<String>,
}
impl Default for BasicHpDetails {
    fn default() -> Self {
        let mut array = Vec::new();
        for _ in 0..6 {
            array.push("Hp_BasicHp".to_owned());
        }
        Self { array }
    }
}
impl BasicHpDetails {
    pub fn count(&self) -> usize {
        self.array.len()
    }
}

fn spawn_health_uis(
    event: On<canum_play::setup::StartSessionBaseUi>,
    mut commands: Commands,
    health: Res<HealthDetails>,
    q_top_left: Query<Entity, With<crate::TopLeft>>,
    q_children: Query<&Children>,
    q_health: Query<(), With<canum_play::health::HealthBar>>,
) {
    let Ok(top_left) = q_top_left.single() else {
        return;
    };
    let Ok(children) = q_children.get(event.player_entity) else {
        return;
    };
    let mut health_entity = None;
    if q_health.get(event.player_entity).is_ok() {
        health_entity = Some(event.player_entity);
    } else {
        for child in children.iter() {
            if q_health.get(child).is_ok() {
                health_entity = Some(child);
                break;
            }
        }
    }
    let Some(health_entity) = health_entity else {
        error!(
            "Player doesn't have a child or self marked with `canum_play::health::HealthBar`, so no health bar UI is spawned."
        );
        return;
    };
    match health.as_ref() {
        HealthDetails::BasicHp(basic_hp) => {
            commands.spawn((
                ChildOf(top_left),
                crate::health::integer_health(health_entity, basic_hp),
            ));
        }
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

pub(crate) fn integer_health(watch: Entity, basic_hp: &BasicHpDetails) -> impl Bundle {
    let mut sub_nodes = Vec::new();
    for animation_name in basic_hp.array.iter() {
        sub_nodes.push((
            Node {
                align_self: AlignSelf::Center,
                width: px(32.0),
                height: px(32.0),
                ..default()
            },
            Animation::new(animation_name, Vec2::new(32.0, 32.0)).with_pause(0),
        ));
    }
    (
        IntegerHealthUi {
            watch,
            ui_count: basic_hp.count(),
        },
        Node {
            align_content: AlignContent::Start,
            justify_content: JustifyContent::Start,
            column_gap: px(4.0),
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
