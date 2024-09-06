use std::{
    cell::UnsafeCell,
    collections::HashMap,
    hash::{BuildHasher, Hash},
};

use crate::{groupex_guard::GroupexGuard, raw_groupex::GROUPEX_SIZE, RawGroupex};

#[derive(Default)]
pub struct GroupexMap<K, V>
where
    K: Eq + Hash,
{
    raw_groupex: RawGroupex,
    map: HashMap<K, UnsafeCell<V>>,
}

impl<K, V> GroupexMap<K, V>
where
    K: Eq + Hash,
{
    pub fn lock(&self, key: K) -> Option<GroupexGuard<'_, V>> {
        let data = self.map.get(&key)?;

        let hash = self.map.hasher().hash_one(key) as usize;
        let index = hash % GROUPEX_SIZE;
        self.raw_groupex.lock(index);

        Some(GroupexGuard::new(&self.raw_groupex, index, data))
    }
}

impl<K, V> From<HashMap<K, V>> for GroupexMap<K, V>
where
    K: Eq + Hash,
{
    fn from(value: HashMap<K, V>) -> Self {
        let map = value
            .into_iter()
            .map(|(k, v)| (k, UnsafeCell::new(v)))
            .collect();

        GroupexMap {
            raw_groupex: RawGroupex::new(),
            map,
        }
    }
}

impl<K, V> Into<HashMap<K, V>> for GroupexMap<K, V>
where
    K: Eq + Hash,
{
    fn into(self) -> HashMap<K, V> {
        self.map
            .into_iter()
            .map(|(k, v)| (k, v.into_inner()))
            .collect()
    }
}

unsafe impl<K, V> Sync for GroupexMap<K, V> where K: Eq + Hash {}
