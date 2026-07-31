use super::*;

pub(super) struct FlipperPlugin;

impl Plugin for FlipperPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<Flipper>()
            .on_insert(
                |mut world, canum_play::prelude::HookContext { entity, .. }| {
                    let font = world.resource::<canum_res::PixelFonts>().normal.clone();
                    world.commands().entity(entity).insert((
                        FlipperCursor {
                            timer: Timer::new(
                                CONFIG.client.text_roll_interval,
                                TimerMode::Repeating,
                            ),
                            ..default()
                        },
                        canum_fx::text::MultilineText::new(
                            canum_res::ImageFontText::default()
                                .font(font)
                                .font_height(14.0),
                        ),
                    ));
                    if let Some(mut node) = world.get_mut::<Node>(entity) {
                        node.flex_direction = FlexDirection::Column;
                        node.align_items = AlignItems::Start;
                    }
                },
            );
        app.add_systems(FixedPreUpdate, handle_keyboard);
        app.add_systems(FixedUpdate, roll_text);
        app.add_observer(handle_input);
    }
}

/// Rolls multiple pages of text, and flips it according to player input.
#[derive(Component, Debug, Default)]
#[require(Node)]
pub(super) struct Flipper {
    pub text: Vec<String>,
}

/// Sent to parent if the flipper has completed all pages.
#[derive(EntityEvent)]
pub(super) struct FlipperComplete {
    pub entity: Entity,
}

/// Stores the current page and text rolling status.
#[derive(Component, Debug, Default)]
struct FlipperCursor {
    timer: Timer,
    page_completed: usize,
    page: usize,
    cursor: usize,
}

fn roll_text(
    commands: ParallelCommands,
    mut q_flipper: Query<(Entity, &Flipper, &mut FlipperCursor)>,
    time: Res<Time>,
) {
    q_flipper
        .par_iter_mut()
        .for_each(|(entity, flipper, mut cursor)| {
            if cursor.page_completed >= flipper.text.len() {
                return;
            }
            if cursor.page == cursor.page_completed
                && cursor.cursor < flipper.text[cursor.page].len()
                && cursor.timer.tick(time.delta()).is_finished()
            {
                let ch = flipper.text[cursor.page][cursor.cursor..]
                    .chars()
                    .next()
                    .unwrap();
                cursor.cursor += ch.len_utf8();
                commands.command_scope(|mut commands| {
                    commands.trigger(canum_fx::text::MultilineTextAppend {
                        entity,
                        append: String::from(ch),
                    });
                })
            }
        });
}

fn handle_input(
    event: On<FlipperInput>,
    mut q_flipper: Query<(Entity, &Flipper, &mut FlipperCursor, &ChildOf)>,
    mut commands: Commands,
) {
    let Ok((entity, flipper, mut cursor, parent)) = q_flipper.get_mut(event.entity) else {
        return;
    };
    if cursor.page_completed >= flipper.text.len() {
        return;
    }
    if cursor.page > cursor.page_completed {
        cursor.page = cursor.page_completed;
    }
    let target_text = &flipper.text[cursor.page];
    match event.kind {
        FlipperInputKind::NextPage => {
            if cursor.page == cursor.page_completed {
                if cursor.cursor < target_text.len() {
                    commands.trigger(canum_fx::text::MultilineTextClear { entity });
                    commands.trigger(canum_fx::text::MultilineTextAppend {
                        entity,
                        append: target_text.clone(),
                    });
                    cursor.cursor = target_text.len();
                } else {
                    cursor.cursor = 0;
                    cursor.page += 1;
                    cursor.page_completed += 1;
                    commands.trigger(canum_fx::text::MultilineTextClear { entity });
                    commands.spawn(canum_res::sound::Sound::new("Ui_TextNext"));
                    if cursor.page_completed == flipper.text.len() {
                        commands.trigger(FlipperComplete { entity: parent.0 });
                    }
                }
            } else {
                cursor.page += 1;
                commands.trigger(canum_fx::text::MultilineTextClear { entity });
                if cursor.page == cursor.page_completed {
                    commands.trigger(canum_fx::text::MultilineTextAppend {
                        entity,
                        append: flipper.text[cursor.page][..cursor.cursor].to_owned(),
                    });
                } else {
                    commands.trigger(canum_fx::text::MultilineTextAppend {
                        entity,
                        append: flipper.text[cursor.page].to_owned(),
                    });
                }
            }
        }
        FlipperInputKind::PrevPage => {
            if cursor.page > 0 {
                cursor.page -= 1;
                commands.trigger(canum_fx::text::MultilineTextAppend {
                    entity,
                    append: flipper.text[cursor.page].to_owned(),
                });
            }
        }
    }
}

#[derive(EntityEvent)]
struct FlipperInput {
    entity: Entity,
    kind: FlipperInputKind,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum FlipperInputKind {
    NextPage,
    PrevPage,
}

fn handle_keyboard(
    mut commands: Commands,
    key: Res<ButtonInput<KeyCode>>,
    q_flipper: Query<Entity, With<Flipper>>,
    save: Res<Save>,
) {
    if key.any_just_pressed([
        KeyCode::Enter,
        save.keyboard.confirm,
        save.keyboard.move_down,
        save.keyboard.move_right,
    ]) {
        for entity in q_flipper.iter() {
            commands.trigger(FlipperInput {
                entity,
                kind: FlipperInputKind::NextPage,
            });
        }
    }
    if key.any_just_pressed([save.keyboard.move_up, save.keyboard.move_left]) {
        for entity in q_flipper.iter() {
            commands.trigger(FlipperInput {
                entity,
                kind: FlipperInputKind::PrevPage,
            });
        }
    }
}
