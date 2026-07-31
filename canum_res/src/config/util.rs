use bevy::prelude::{Deref, DerefMut};
use serde::de;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct StringList(pub String);

impl StringList {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<'de> Deserialize<'de> for StringList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringListVisitor;

        impl<'de> de::Visitor<'de> for StringListVisitor {
            type Value = String;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a string or a sequence of strings")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<String, E> {
                Ok(v.to_owned())
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<String, E> {
                Ok(v)
            }

            fn visit_borrowed_str<E: de::Error>(self, v: &'de str) -> Result<String, E> {
                Ok(v.to_owned())
            }

            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<String, A::Error> {
                let mut out = String::new();
                while let Some(s) = seq.next_element::<String>()? {
                    out.push_str(&s);
                }
                Ok(out)
            }
        }

        let value = deserializer.deserialize_any(StringListVisitor)?;
        Ok(StringList(value))
    }
}
