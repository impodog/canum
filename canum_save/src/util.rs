use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    collections::{BTreeMap, btree_map},
};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BTreeMultiSet(BTreeMap<String, usize>);

impl BTreeMultiSet {
    pub fn insert(&mut self, value: impl Into<String>) {
        *self.0.entry(value.into()).or_default() += 1;
    }

    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        String: Borrow<Q>,
        Q: Ord + ?Sized,
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
        Q: Ord + ?Sized,
    {
        self.0.contains_key(value)
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Gets an iterator over multiset values falling into the given range.
    pub fn range<'a, Q, R>(&'a self, range: R) -> Range<'a>
    where
        String: Borrow<Q>,
        Q: Ord + ?Sized,
        R: std::ops::RangeBounds<Q>,
    {
        Range {
            internal: self.0.range(range),
            current: None,
            remaining: 0,
        }
    }

    /// Gets an iterator over multiset values whose prefix is the given string.
    pub fn range_starting_with<'a>(&'a self, prefix: impl Into<String>) -> Range<'a> {
        let prefix = prefix.into();
        let prefix_end = format!("{prefix}~");
        self.range(prefix..=prefix_end)
    }
}

#[derive(Debug, Clone)]
pub struct Iter<'a> {
    internal: btree_map::Iter<'a, String, usize>,
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

#[derive(Debug, Clone)]
pub struct Range<'a> {
    internal: btree_map::Range<'a, String, usize>,
    current: Option<&'a String>,
    remaining: usize,
}

impl<'a> Iterator for Range<'a> {
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

/// Stores either integer or float. Supports addition and subtraction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AnyValue {
    Integer(i64),
    Float(f64),
}
impl Default for AnyValue {
    fn default() -> Self {
        Self::Integer(0)
    }
}

macro_rules! implement_integer {
    ($self: ty, $rhs: ty) => {
        impl std::ops::AddAssign<$rhs> for $self {
            fn add_assign(&mut self, rhs: $rhs) {
                match self {
                    Self::Integer(lhs) => *lhs += rhs as i64,
                    Self::Float(lhs) => *lhs += rhs as f64,
                }
            }
        }
        impl std::ops::SubAssign<$rhs> for $self {
            fn sub_assign(&mut self, rhs: $rhs) {
                match self {
                    Self::Integer(lhs) => *lhs -= rhs as i64,
                    Self::Float(lhs) => *lhs -= rhs as f64,
                }
            }
        }
    };
}
macro_rules! implement_float {
    ($self: ty, $rhs: ty) => {
        impl std::ops::AddAssign<$rhs> for $self {
            fn add_assign(&mut self, rhs: $rhs) {
                let result = match self {
                    Self::Integer(lhs) => *lhs as f64 + rhs as f64,
                    Self::Float(lhs) => *lhs + rhs as f64,
                };
                *self = Self::Float(result);
            }
        }
        impl std::ops::SubAssign<$rhs> for $self {
            fn sub_assign(&mut self, rhs: $rhs) {
                let result = match self {
                    Self::Integer(lhs) => *lhs as f64 - rhs as f64,
                    Self::Float(lhs) => *lhs - rhs as f64,
                };
                *self = Self::Float(result);
            }
        }
    };
}

implement_integer!(AnyValue, i8);
implement_integer!(AnyValue, i16);
implement_integer!(AnyValue, i32);
implement_integer!(AnyValue, i64);
implement_float!(AnyValue, f32);
implement_float!(AnyValue, f64);
