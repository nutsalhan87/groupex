use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use super::Groupex;

pub struct GroupexGuard<'a, GROUPEX: Groupex, T> {
    groupex: &'a GROUPEX,
    index: usize,
    data: &'a UnsafeCell<T>,
}

impl<'a, GROUPEX: Groupex, T> GroupexGuard<'a, GROUPEX, T> {
    pub(crate) fn new(
        groupex: &'a GROUPEX,
        index: usize,
        data: &'a UnsafeCell<T>,
    ) -> Self {
        GroupexGuard {
            groupex,
            index,
            data,
        }
    }
}

impl<GROUPEX: Groupex, T> Deref for GroupexGuard<'_, GROUPEX, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.get() }
    }
}

impl<GROUPEX: Groupex, T> DerefMut for GroupexGuard<'_, GROUPEX, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data.get() }
    }
}

impl<GROUPEX: Groupex, T> Drop for GroupexGuard<'_, GROUPEX, T> {
    fn drop(&mut self) {
        self.groupex.unlock(self.index);
    }
}
