use sep_40_oracle::Asset;
use soroban_sdk::{contracttype, Env, Map, Vec};

// NB: It seemed obvious that implementing a generic `Set<T>` would work, but alas, #[contractimpl]
// macro appears to be incompatible with generic parameters.
// TODO: Check if there are known implementations of `Set<T>` in 3rd party crates

#[contracttype]
/// A set of [`Asset`]'s
pub struct AssetsSet(Map<Asset, ()>);

impl AssetsSet {
    pub fn new(e: &Env) -> Self {
        Self(Map::new(e))
    }

    /// # Returns
    ///  `true` if the set contains a specified entry. Otherwise returns `false`
    pub fn contains(&self, entry: Asset) -> bool {
        self.0.contains_key(entry)
    }

    /// Inserts a specified entry into the set. If a specified entry already exists in the set, no
    /// effect takes place
    pub fn insert(&mut self, entry: Asset) {
        self.0.set(entry, ());
    }

    /// Removes the corresponding entry from the set.
    ///
    /// # Returns
    ///  `None` if the set does not contain the specified entry
    pub fn remove(&mut self, entry: Asset) -> Option<()> {
        self.0.remove(entry)
    }

    /// Returns a [`Vec`] of all keys in the map
    pub fn entries(&self) -> Vec<Asset> {
        self.0.keys()
    }

    /// # Returns
    ///  `true` if the set contains no entries. Otherwise returns `false`
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// # Returns
    ///  [`u32`] amount of unique entries in the set
    pub fn len(&self) -> u32 {
        self.0.len()
    }
}
