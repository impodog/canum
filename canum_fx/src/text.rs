use crate::prelude::*;

pub(super) struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(update_multiline_text)
            .add_observer(clear_multiline_text);
        app.world_mut()
            .register_component_hooks::<MultilineText>()
            .on_add(
                |mut world, bevy::ecs::lifecycle::HookContext { entity, .. }| {
                    let text = if let Some(text) = world.get::<MultilineText>(entity) {
                        text.text.text.clone()
                    } else {
                        warn!("MultilineText is spawned, but no component found");
                        return;
                    };
                    world.commands().trigger(MultilineTextAppend {
                        entity,
                        append: text,
                    });
                },
            );
    }
}

/// Adaptor for `bevy_image_font::ImageFontText`, a UI node, that allows the '\n' character.
/// All new line children are with the same configuration as the parent.
///
/// ## Note
///
/// To ensure correct behavior, this entity's Node must be flex in columns.
#[derive(Component, Default, Debug, Clone)]
#[require(Node)]
pub struct MultilineText {
    pub text: canum_res::ImageFontText,
    lines: Vec<Entity>,
    length: usize,
}

impl MultilineText {
    pub fn new(text: canum_res::ImageFontText) -> Self {
        Self {
            text,
            lines: default(),
            length: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

#[derive(EntityEvent, Debug)]
pub struct MultilineTextAppend {
    pub entity: Entity,
    pub append: String,
}

fn update_multiline_text(
    event: On<MultilineTextAppend>,
    mut commands: Commands,
    mut q_text: Query<&mut MultilineText>,
    mut q_subtext: Query<&mut canum_res::ImageFontText>,
) {
    let Ok(mut text) = q_text.get_mut(event.entity) else {
        return;
    };
    let mut lines = event.append.split('\n');

    if let Some(last) = text.lines.last() {
        let Some(first_line) = lines.next() else {
            return;
        };
        let Ok(mut last) = q_subtext.get_mut(*last) else {
            return;
        };
        last.text.push_str(first_line);
    }

    for line in lines {
        let child = commands
            .spawn((
                ChildOf(event.entity),
                Node {
                    flex_shrink: 0.0,
                    ..default()
                },
                ImageNode::default(),
                canum_res::ImageFontText::default()
                    .text(line)
                    .font(text.text.font.clone())
                    .font_height(text.text.font_height),
            ))
            .id();
        text.lines.push(child);
    }

    text.length += event.append.len();
}

#[derive(EntityEvent, Debug)]
pub struct MultilineTextClear {
    pub entity: Entity,
}

fn clear_multiline_text(
    event: On<MultilineTextClear>,
    mut commands: Commands,
    mut q_text: Query<&mut MultilineText>,
) {
    let Ok(mut text) = q_text.get_mut(event.entity) else {
        return;
    };
    text.length = 0;
    text.text.text.clear();
    for line_entity in text.lines.drain(..) {
        commands.entity(line_entity).despawn();
    }
}
