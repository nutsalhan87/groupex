use std::cell::UnsafeCell;

use crate::{guard::GroupexGuard, Groupex};

#[derive(Default)]
pub struct GroupexVec<G: Groupex, T> {
    groupex: G,
    vec: Vec<UnsafeCell<T>>,
}

impl<G: Groupex, T> GroupexVec<G, T> {
    pub fn lock(&self, index: usize) -> Option<GroupexGuard<'_, G, T>> {
        let data = self.vec.get(index)?;

        let index = index % self.groupex.elements();
        self.groupex.lock(index);

        Some(GroupexGuard::new(&self.groupex, index, data))
    }
}

impl<G: Groupex, T> From<Vec<T>> for GroupexVec<G, T> {
    fn from(value: Vec<T>) -> Self {
        let vec = value.into_iter().map(UnsafeCell::new).collect();

        GroupexVec {
            groupex: G::new(),
            vec,
        }
    }
}

impl<G: Groupex, T> Into<Vec<T>> for GroupexVec<G, T> {
    fn into(self) -> Vec<T> {
        self.vec
            .into_iter()
            .map(|v| v.into_inner())
            .collect()
    }
}

unsafe impl<G: Groupex, T> Sync for GroupexVec<G, T> {}

unsafe impl<G: Groupex, T> Send for GroupexVec<G, T> {}
