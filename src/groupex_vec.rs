use std::cell::UnsafeCell;

use crate::{GroupexGuard, RawGroupex};

#[derive(Default)]
pub struct GroupexVec<const BLOCKS: usize, T> {
    raw_groupex: RawGroupex<BLOCKS>,
    vec: Vec<UnsafeCell<T>>,
}

impl<const BLOCKS: usize, T> GroupexVec<BLOCKS, T> {
    pub fn lock(&self, index: usize) -> Option<GroupexGuard<'_, BLOCKS, T>> {
        let data = self.vec.get(index)?;

        let index = index % self.raw_groupex.elements();
        self.raw_groupex.lock(index);

        Some(GroupexGuard::new(&self.raw_groupex, index, data))
    }
}

impl<const BLOCKS: usize, T> From<Vec<T>> for GroupexVec<BLOCKS, T> {
    fn from(value: Vec<T>) -> Self {
        let vec = value.into_iter().map(UnsafeCell::new).collect();

        GroupexVec {
            raw_groupex: RawGroupex::new(),
            vec,
        }
    }
}

impl<const BLOCKS: usize, T> Into<Vec<T>> for GroupexVec<BLOCKS, T> {
    fn into(self) -> Vec<T> {
        self.vec
            .into_iter()
            .map(|v| v.into_inner())
            .collect()
    }
}

unsafe impl<const BLOCKS: usize, T> Sync for GroupexVec<BLOCKS, T> {}
