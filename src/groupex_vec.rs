use std::cell::UnsafeCell;

use crate::{raw_groupex::GROUPEX_SIZE, GroupexGuard, RawGroupex};

#[derive(Default)]
pub struct GroupexVec<T> {
    raw_groupex: RawGroupex,
    vec: Vec<UnsafeCell<T>>,
}

impl<T> GroupexVec<T> {
    pub fn lock(&self, index: usize) -> Option<GroupexGuard<'_, T>> {
        let data = self.vec.get(index)?;

        let index = index % GROUPEX_SIZE;
        self.raw_groupex.lock(index);

        Some(GroupexGuard::new(&self.raw_groupex, index, data))
    }
}

impl<T> From<Vec<T>> for GroupexVec<T> {
    fn from(value: Vec<T>) -> Self {
        let vec = value.into_iter().map(UnsafeCell::new).collect();

        GroupexVec {
            raw_groupex: RawGroupex::new(),
            vec,
        }
    }
}

impl<T> Into<Vec<T>> for GroupexVec<T> {
    fn into(self) -> Vec<T> {
        self.vec
            .into_iter()
            .map(|v| v.into_inner())
            .collect()
    }
}

unsafe impl<T> Sync for GroupexVec<T> {}
