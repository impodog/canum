use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    collections::{HashMap, hash_map},
    hash::Hash,
};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct HashMultiSet(HashMap<String, usize>);

impl HashMultiSet {
    pub fn insert(&mut self, value: impl Into<String>) {
        *self.0.entry(value.into()).or_default() += 1;
    }

    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        String: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        if let Some(count) = self.0.get_mut(value) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.0.remove(value);
            }
            true
        } else {
            false
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a> {
        Iter {
            internal: self.0.iter(),
            current: None,
            remaining: 0,
        }
    }

    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        String: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.0.contains_key(value)
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

pub struct Iter<'a> {
    internal: hash_map::Iter<'a, String, usize>,
    current: Option<&'a String>,
    remaining: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if let Some(current) = self.current {
            self.remaining = self.remaining.saturating_sub(1);
            Some(current)
        } else {
            if let Some((next, count)) = self.internal.next() {
                self.current = Some(next);
                self.remaining = count.saturating_sub(1);
                Some(next)
            } else {
                None
            }
        };
        if self.remaining == 0 {
            self.current = None;
        }
        result
    }
}
