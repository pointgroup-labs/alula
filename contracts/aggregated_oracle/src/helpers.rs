use soroban_sdk::{Env, IntoVal, Map, TryFromVal, Val, Vec};

pub struct Set<T>(Map<T, ()>);

impl<T> Set<T>
where
    T: IntoVal<Env, Val> + TryFromVal<Env, Val>,
{
    pub fn new(e: &Env) -> Self {
        Set(Map::new(e))
    }

    pub fn contains(&self, k: T) -> bool {
        self.0.contains_key(k)
    }

    pub fn insert(&mut self, k: T) {
        self.0.set(k, ());
    }

    /// Remove the corresponding entry from the set.
    ///
    /// Returns `None` if the set does not contain the specified entry
    pub fn remove(&mut self, k: T) -> Option<()> {
        self.0.remove(k)
    }

    /// Returns a [`Vec`] of all keys in the map
    pub fn entries(&self) -> Vec<T> {
        self.0.keys()
    }
}

impl<T> Set<T> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> u32 {
        self.0.len()
    }
}
