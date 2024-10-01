use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use super::RawSizedGroupexParkingLot;

pub struct SizedGroupexParkingLotGuard<'a, const BLOCKS: usize, T> {
    raw_groupex: &'a RawSizedGroupexParkingLot<BLOCKS>,
    index: usize,
    data: &'a UnsafeCell<T>,
}

impl<'a, const BLOCKS: usize, T> SizedGroupexParkingLotGuard<'a, BLOCKS, T> {
    pub(crate) fn new(
        raw_groupex: &'a RawSizedGroupexParkingLot<BLOCKS>,
        index: usize,
        data: &'a UnsafeCell<T>,
    ) -> Self {
        SizedGroupexParkingLotGuard {
            raw_groupex,
            index,
            data,
        }
    }
}

impl<const BLOCKS: usize, T> Deref for SizedGroupexParkingLotGuard<'_, BLOCKS, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.get() }
    }
}

impl<const BLOCKS: usize, T> DerefMut for SizedGroupexParkingLotGuard<'_, BLOCKS, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data.get() }
    }
}

impl<const BLOCKS: usize, T> Drop for SizedGroupexParkingLotGuard<'_, BLOCKS, T> {
    fn drop(&mut self) {
        self.raw_groupex.unlock(self.index);
    }
}
