use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use crate::RawGroupex;

pub struct GroupexGuard<'a, T> {
    raw_groupex: &'a RawGroupex,
    index: usize,
    data: &'a UnsafeCell<T>,
}

impl<'a, T> GroupexGuard<'a, T> {
    pub(crate) fn new(
        raw_groupex: &'a RawGroupex,
        index: usize,
        data: &'a UnsafeCell<T>,
    ) -> Self {
        GroupexGuard {
            raw_groupex,
            index,
            data,
        }
    }
}

impl<T> Deref for GroupexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.get() }
    }
}

impl<T> DerefMut for GroupexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data.get() }
    }
}

impl<T> Drop for GroupexGuard<'_, T> {
    fn drop(&mut self) {
        self.raw_groupex.unlock(self.index);
    }
}
