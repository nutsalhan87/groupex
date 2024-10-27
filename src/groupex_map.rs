use std::{
    cell::UnsafeCell,
    collections::HashMap,
    hash::{BuildHasher, Hash},
};

use crate::{guard::GroupexGuard, Groupex};

#[derive(Default)]
pub struct GroupexMap<G, K, V>
where
    G: Groupex,
    K: Eq + Hash,
{
    groupex: G,
    map: HashMap<K, UnsafeCell<V>>,
}

impl<G, K, V> GroupexMap<G, K, V>
where
    G: Groupex,
    K: Eq + Hash,
{
    pub fn lock(&self, key: K) -> Option<GroupexGuard<'_, G, V>> {
        let data = self.map.get(&key)?;

        let hash = self.map.hasher().hash_one(key) as usize;
        let index = hash % self.groupex.elements();
        self.groupex.lock(index);

        Some(GroupexGuard::new(&self.groupex, index, data))
    }
}

impl<G, K, V> From<HashMap<K, V>> for GroupexMap<G, K, V>
where
    G: Groupex,
    K: Eq + Hash,
{
    fn from(value: HashMap<K, V>) -> Self {
        let map = value
            .into_iter()
            .map(|(k, v)| (k, UnsafeCell::new(v)))
            .collect();

        GroupexMap {
            groupex: G::new(),
            map,
        }
    }
}

impl<G, K, V> Into<HashMap<K, V>> for GroupexMap<G, K, V>
where
    G: Groupex,
    K: Eq + Hash,
{
    fn into(self) -> HashMap<K, V> {
        self.map
            .into_iter()
            .map(|(k, v)| (k, v.into_inner()))
            .collect()
    }
}

unsafe impl<G, K, V> Sync for GroupexMap<G, K, V>
where
    G: Groupex,
    K: Eq + Hash,
{
}

unsafe impl<G, K, V> Send for GroupexMap<G, K, V>
where
    G: Groupex,
    K: Eq + Hash,
{
}