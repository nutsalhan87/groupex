use std::{
    cell::UnsafeCell,
    collections::HashMap,
    hash::{BuildHasher, Hash},
};

use crate::{SizedGroupexParkingLotGuard, RawSizedGroupexParkingLot};

#[derive(Default)]
pub struct GroupexMap3<const BLOCKS: usize, K, V>
where
    K: Eq + Hash,
{
    raw_groupex: RawSizedGroupexParkingLot<BLOCKS>,
    map: HashMap<K, UnsafeCell<V>>,
}

impl<const BLOCKS: usize, K, V> GroupexMap3<BLOCKS, K, V>
where
    K: Eq + Hash,
{
    pub fn lock(&self, key: K) -> Option<SizedGroupexParkingLotGuard<'_, BLOCKS, V>> {
        let data = self.map.get(&key)?;

        let hash = self.map.hasher().hash_one(key) as usize;
        let index = hash % self.raw_groupex.elements();
        self.raw_groupex.lock(index);

        Some(SizedGroupexParkingLotGuard::new(&self.raw_groupex, index, data))
    }
}

impl<const BLOCKS: usize, K, V> From<HashMap<K, V>> for GroupexMap3<BLOCKS, K, V>
where
    K: Eq + Hash,
{
    fn from(value: HashMap<K, V>) -> Self {
        let map = value
            .into_iter()
            .map(|(k, v)| (k, UnsafeCell::new(v)))
            .collect();

        GroupexMap3 {
            raw_groupex: RawSizedGroupexParkingLot::new(),
            map,
        }
    }
}

impl<const BLOCKS: usize, K, V> Into<HashMap<K, V>> for GroupexMap3<BLOCKS, K, V>
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

unsafe impl<const BLOCKS: usize, K, V> Sync for GroupexMap3<BLOCKS, K, V> where K: Eq + Hash {}
