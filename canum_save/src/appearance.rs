use std::{borrow::Borrow, collections::HashMap, hash::Hash};

use crate::*;

/// User customizable appearances.
#[derive(Debug, Serialize, Deserialize)]
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

/// Map of all text, language selected by the player.
#[derive(Resource, Default, Debug, Clone)]
pub struct Lang(pub HashMap<String, String>);

impl Lang {
    /// Gets the text with the key, or a placeholder if none.
    pub fn get<T>(&self, key: &T) -> &str
    where
        String: Borrow<T>,
        T: Ord + Hash + ?Sized,
    {
        self.0
            .get(key)
            .map(String::as_str)
            .unwrap_or("<MISSING TEXT>")
    }
}
