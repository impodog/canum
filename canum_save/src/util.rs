use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    collections::{BTreeMap, btree_map},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BTreeMultiSet<K: Ord = String>(BTreeMap<K, usize>);

impl<K: Ord> Default for BTreeMultiSet<K> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

impl<K: Ord> BTreeMultiSet<K> {
    pub fn insert(&mut self, value: impl Into<K>) {
        *self.0.entry(value.into()).or_default() += 1;
    }

    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        K: Borrow<Q>,
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

    pub fn iter<'a>(&'a self) -> Iter<'a, K> {
        Iter {
            internal: self.0.iter(),
            current: None,
            remaining: 0,
        }
    }

    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.0.contains_key(value)
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Gets an iterator over multiset values falling into the given range.
    pub fn range<'a, Q, R>(&'a self, range: R) -> Range<'a, K>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        R: std::ops::RangeBounds<Q>,
    {
        Range {
            internal: self.0.range(range),
            current: None,
            remaining: 0,
        }
    }
}

impl BTreeMultiSet<String> {
    /// Gets an iterator over multiset values whose prefix is the given string.
    pub fn range_starting_with<'a>(&'a self, prefix: impl Into<String>) -> Range<'a, String> {
        let prefix = prefix.into();
        let prefix_end = format!("{prefix}~");
        self.range(prefix..=prefix_end)
    }
}

#[derive(Debug, Clone)]
pub struct Iter<'a, K: Ord> {
    internal: btree_map::Iter<'a, K, usize>,
    current: Option<&'a K>,
    remaining: usize,
}

impl<'a, K: Ord> Iterator for Iter<'a, K> {
    type Item = &'a K;

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
pub struct Range<'a, K: Ord> {
    internal: btree_map::Range<'a, K, usize>,
    current: Option<&'a K>,
    remaining: usize,
}

impl<'a, K: Ord> Iterator for Range<'a, K> {
    type Item = &'a K;

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
        impl std::cmp::PartialEq<$rhs> for AnyValue {
            fn eq(&self, other: &$rhs) -> bool {
                match self {
                    Self::Integer(lhs) => *lhs == *other as i64,
                    Self::Float(lhs) => *lhs == *other as f64,
                }
            }
        }
        impl std::cmp::PartialOrd<$rhs> for AnyValue {
            fn partial_cmp(&self, other: &$rhs) -> Option<std::cmp::Ordering> {
                match self {
                    Self::Integer(lhs) => lhs.partial_cmp(&(*other as i64)),
                    Self::Float(lhs) => lhs.partial_cmp(&(*other as f64)),
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
        impl std::cmp::PartialEq<$rhs> for AnyValue {
            fn eq(&self, other: &$rhs) -> bool {
                match self {
                    Self::Integer(lhs) => (*lhs as $rhs - *other).abs() < 1e-10,
                    Self::Float(lhs) => (*lhs - *other as f64).abs() < 1e-10,
                }
            }
        }
        impl std::cmp::PartialOrd<$rhs> for AnyValue {
            fn partial_cmp(&self, other: &$rhs) -> Option<std::cmp::Ordering> {
                match self {
                    Self::Integer(lhs) => (*lhs as $rhs - *other).partial_cmp(&0.0),
                    Self::Float(lhs) => (*lhs - *other as f64).partial_cmp(&0.0),
                }
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
