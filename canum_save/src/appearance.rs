use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::Hash,
    sync::{LazyLock, RwLock},
};

use crate::*;

pub(super) struct AppearancePlugin;

impl Plugin for AppearancePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(update_appearance);
        app.add_systems(PostStartup, update_appearance_on_startup);
    }
}

/// User customizable appearances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appearance {
    pub player: String,
    pub language: String,
}
impl Default for Appearance {
    fn default() -> Self {
        Self {
            player: "Cyan".to_owned(),
            language: "en".to_owned(),
        }
    }
}

static APPEARANCE_INFO: LazyLock<RwLock<Appearance>> = LazyLock::new(|| RwLock::new(default()));

/// Only when this event happens, all appearance info is updated.
#[derive(Event, Default)]
pub struct UpdateAppearance;

fn update_appearance(_event: On<UpdateAppearance>, save: Res<Save>) {
    let mut appearance = APPEARANCE_INFO.write().unwrap();
    *appearance = save.appearance.clone();
}
fn update_appearance_on_startup(mut commands: Commands) {
    commands.trigger(UpdateAppearance);
}

/// Map of all text, language selected by the player.
#[derive(Resource, Default, Debug, Clone)]
pub struct Lang(pub HashMap<String, String>);

impl Lang {
    fn unfold_text(text: &str) -> String {
        let mut result = String::new();
        let mut index = 0;
        let mut front = usize::MAX;

        let lock = APPEARANCE_INFO.read().unwrap();

        for ch in text.chars() {
            if ch == '<' {
                front = index + 1;
            } else if front != usize::MAX {
                if ch == '>' {
                    let name = &text[front..index];
                    match name {
                        "PLAYER" => result.push_str(&lock.player),
                        "LANGUAGE" => result.push_str(&lock.language),
                        _ => result.push_str(&text[front - 1..index + 1]),
                    }
                    front = usize::MAX;
                } else if !(ch.is_ascii_alphanumeric() || ch == '_') {
                    result.push_str(&text[front - 1..index + ch.len_utf8()]);
                    front = usize::MAX;
                }
            } else {
                result.push(ch);
            }
            index += ch.len_utf8();
        }

        result
    }

    /// Gets the text with the key, or a placeholder if none.
    pub fn get<'s, T>(&'s self, key: &T) -> &'s str
    where
        String: Borrow<T>,
        T: Ord + Hash + ?Sized,
    {
        self.0
            .get(key)
            .map(String::as_str)
            .unwrap_or("<MISSING TEXT>")
    }

    /// Gets the text with the key, or an empty str if none.
    pub fn get_or_empty<T>(&self, key: &T) -> &str
    where
        String: Borrow<T>,
        T: Ord + Hash + ?Sized,
    {
        self.0.get(key).map(String::as_str).unwrap_or("")
    }

    pub fn get_special<T>(&self, key: &T) -> String
    where
        String: Borrow<T>,
        T: Ord + Hash + ?Sized,
    {
        Self::unfold_text(self.get(key))
    }
}
