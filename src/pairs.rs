use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UnorderedPair<T>(pub T, pub T);

impl<T> UnorderedPair<T> {
    pub fn contains(&self, item: T) -> bool
    where
        T: PartialEq,
    {
        [&self.0, &self.1].into_iter().any(move |i| *i == item)
    }
}

impl<T> From<(T, T)> for UnorderedPair<T> {
    fn from((a, b): (T, T)) -> Self {
        Self(a, b)
    }
}

impl<T: PartialEq> PartialEq for UnorderedPair<T> {
    fn eq(&self, other: &Self) -> bool {
        (self.0 == other.0 && self.1 == other.1) || (self.0 == other.1 && self.1 == other.0)
    }
}

impl<T: Eq> Eq for UnorderedPair<T> {}

impl<T: Hash + PartialOrd> Hash for UnorderedPair<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for i in {
            if self.0 < self.1 {
                [&self.0, &self.1]
            } else {
                [&self.1, &self.0]
            }
        } {
            i.hash(state);
        }
    }
}

pub type UnorderedPairs<T> = HashSet<UnorderedPair<T>>;
