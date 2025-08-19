use sep_40_oracle::Asset;
use soroban_sdk::{contracttype, Env, Map, Vec};

#[contracttype]
pub struct AssetsSet(Map<Asset, ()>);

impl AssetsSet {
    pub fn new(e: &Env) -> Self {
        Self(Map::new(e))
    }

    pub fn contains(&self, k: Asset) -> bool {
        self.0.contains_key(k)
    }

    pub fn insert(&mut self, k: Asset) {
        self.0.set(k, ());
    }

    /// Remove the corresponding entry from the set.
    ///
    /// Returns `None` if the set does not contain the specified entry
    pub fn remove(&mut self, k: Asset) -> Option<()> {
        self.0.remove(k)
    }

    /// Returns a [`Vec`] of all keys in the map
    pub fn entries(&self) -> Vec<Asset> {
        self.0.keys()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> u32 {
        self.0.len()
    }
}
