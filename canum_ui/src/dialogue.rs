mod flipper;

use crate::prelude::*;

pub(super) struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((flipper::FlipperPlugin,));
        app.add_systems(FixedPreUpdate, init_dialogue);
        app.add_observer(load_dialogue_section);
    }
}

#[derive(Debug, Default)]
pub struct DialogueSection {
    pub title: String,
    pub text: Vec<String>,
    pub icon_image: String,
}

/// Plays the whole dialogue at a given ui location. There can't be more than one dialogue.
#[derive(Component, Debug, Default)]
#[require(Node, canum_play::controls::OverrideMainControls)]
pub struct Dialogue {
    pub sections: Vec<DialogueSection>,
    pub background: String,
    pub index: usize,
}

impl Dialogue {
    /// Construct a dialogue according a given configuration and the current language data.
    /// Returns default value if config does not exist.
    pub fn from_config(name: &'static str, lang: impl AsRef<canum_save::Lang>) -> Self {
        let Some(config) = CONFIG.assets.dialogue.get(name) else {
            return default();
        };
        let lang = lang.as_ref();
        let mut sections = Vec::new();
        for section in config.sections.iter() {
            let value = lang.get(section);
            let (image, value) = value.split_once(':').unwrap_or(("Empty", value));
            let (title, value) = value.split_once(':').unwrap_or(("", value));
            sections.push(DialogueSection {
                title: title.to_owned(),
                text: value.split("</br>").map(ToOwned::to_owned).collect(),
                icon_image: image.to_owned(),
            });
        }
        Self {
            sections,
            background: config.background.clone(),
            index: 0,
        }
    }
}

/// Sent globally when the dialogue is complete.
#[derive(Event, Default)]
pub struct DialogueComplete;

const DIALOGUE_SIZE: Vec2 = vec2(750.0, 90.0);
const TITLE_SIZE: f32 = 14.0;
const ICON_SIZE: f32 = 64.0;

fn init_dialogue(
    q_dialogue: Query<(Entity, &Dialogue), Added<Dialogue>>,
    mut commands: Commands,
    fonts: Res<canum_res::PixelFonts>,
) {
    for (entity, dialogue) in q_dialogue.iter() {
        let first_image = dialogue
            .sections
            .first()
            .map(|section| section.icon_image.clone())
            .unwrap_or_else(|| "Ui_Dialogue_Background".to_owned());
        commands.entity(entity).insert((
            Node {
                margin: UiRect::all(Val::ZERO),
                width: px(DIALOGUE_SIZE.x),
                height: px(DIALOGUE_SIZE.y),
                padding: UiRect::all(px(3)),
                ..default()
            },
            Animation::new(dialogue.background.clone(), DIALOGUE_SIZE),
        ));
        commands.spawn((
            ChildOf(entity),
            DialogueTitle,
            Node {
                position_type: PositionType::Absolute,
                max_width: px(DIALOGUE_SIZE.x),
                ..default()
            },
            ImageNode::default(),
            canum_res::ImageFontText::default()
                .font(fonts.normal.clone())
                .font_height(14.0),
        ));
        commands.spawn((
            ChildOf(entity),
            Node {
                position_type: PositionType::Absolute,
                max_width: px(DIALOGUE_SIZE.x),
                top: px(TITLE_SIZE + 3.0),
                left: px(ICON_SIZE + 3.0),
                ..default()
            },
            flipper::Flipper::default(),
        ));
        commands.spawn((
            ChildOf(entity),
            DialogueIcon,
            Node {
                position_type: PositionType::Absolute,
                width: px(ICON_SIZE),
                height: px(ICON_SIZE),
                top: px(TITLE_SIZE),
                ..default()
            },
            Animation::new(first_image, vec2(ICON_SIZE, ICON_SIZE)),
        ));
        commands.trigger(flipper::FlipperComplete { entity });
    }
}

/// Marks the title UI node on top of the dialogue box.
#[derive(Component, Default)]
struct DialogueTitle;

/// Marks the icon UI node on left of the dialogue box.
#[derive(Component, Default)]
struct DialogueIcon;

fn load_dialogue_section(
    event: On<flipper::FlipperComplete>,
    mut commands: Commands,
    mut q_dialogue: Query<(&mut Dialogue, &Children)>,
    mut q_title: Query<&mut canum_res::ImageFontText, With<DialogueTitle>>,
    q_flipper: Query<Entity, With<flipper::Flipper>>,
    mut q_icon: Query<&mut Animation, With<DialogueIcon>>,
) {
    let Ok((mut dialogue, children)) = q_dialogue.get_mut(event.entity) else {
        return;
    };
    dialogue.index += 1;
    let Some(section) = dialogue.sections.get(dialogue.index - 1) else {
        commands.trigger(DialogueComplete);
        commands.entity(event.entity).despawn();
        return;
    };
    for child in children.iter() {
        if let Ok(mut text) = q_title.get_mut(child) {
            text.text = section.title.clone();
        } else if let Ok(flipper_entity) = q_flipper.get(child) {
            commands.entity(flipper_entity).insert(flipper::Flipper {
                text: section.text.clone(),
            });
        } else if let Ok(mut animation) = q_icon.get_mut(child) {
            animation.name = section.icon_image.clone();
        }
    }
}
