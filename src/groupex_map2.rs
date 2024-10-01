use std::{
    cell::UnsafeCell,
    collections::HashMap,
    hash::{BuildHasher, Hash},
};

use crate::{SizedGroupexGuard, RawSizedGroupex};

#[derive(Default)]
pub struct GroupexMap2<const BLOCKS: usize, K, V>
where
    K: Eq + Hash,
{
    raw_groupex: RawSizedGroupex<BLOCKS>,
    map: HashMap<K, UnsafeCell<V>>,
}

impl<const BLOCKS: usize, K, V> GroupexMap2<BLOCKS, K, V>
where
    K: Eq + Hash,
{
    pub fn lock(&self, key: K) -> Option<SizedGroupexGuard<'_, BLOCKS, V>> {
        let data = self.map.get(&key)?;

        let hash = self.map.hasher().hash_one(key) as usize;
        let index = hash % self.raw_groupex.elements();
        self.raw_groupex.lock(index);

        Some(SizedGroupexGuard::new(&self.raw_groupex, index, data))
    }
}

impl<const BLOCKS: usize, K, V> From<HashMap<K, V>> for GroupexMap2<BLOCKS, K, V>
where
    K: Eq + Hash,
{
    fn from(value: HashMap<K, V>) -> Self {
        let map = value
            .into_iter()
            .map(|(k, v)| (k, UnsafeCell::new(v)))
            .collect();

        GroupexMap2 {
            raw_groupex: RawSizedGroupex::new(),
            map,
        }
    }
}

impl<const BLOCKS: usize, K, V> Into<HashMap<K, V>> for GroupexMap2<BLOCKS, K, V>
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

unsafe impl<const BLOCKS: usize, K, V> Sync for GroupexMap2<BLOCKS, K, V> where K: Eq + Hash {}
