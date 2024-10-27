use parking_lot_core::{
    park, unpark_all, ParkResult, ParkToken, UnparkToken,
};

use std::{
    hint,
    mem::size_of,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{groupex::get_mask, Groupex};

const SPIN_LIMIT: usize = 5;
const BLOCK_SIZE: usize = size_of::<AtomicU32>() * 8;
const BLOCK_INIT: AtomicU32 = AtomicU32::new(0);

pub struct RawGroupex<const BLOCKS: usize> {
    blocks: [AtomicU32; BLOCKS],
}

impl<const BLOCKS: usize> RawGroupex<BLOCKS> {
    #[inline]
    fn validate_index(&self, index: usize) {
        if index >= BLOCKS * BLOCK_SIZE {
            panic!("Index out of range: must be in [0; {}] but it is {}", BLOCKS * BLOCK_SIZE - 1, index);
        }
    }

    #[cold]
    fn lock_slow(&self, index: usize, mask: u32) {
        let block_index = index / BLOCK_SIZE;

        for spin_cnt in 0..SPIN_LIMIT {
            let prev_block = self.blocks[block_index].fetch_or(mask, Ordering::Acquire);
            if (prev_block | mask) != prev_block {
                return;
            }
            for _ in 0..(1 << spin_cnt) {
                hint::spin_loop();
            }
        }

        loop {
            match unsafe {
                park(
                    &self.blocks[block_index] as *const _ as usize,
                    || {
                        let prev_block =
                            self.blocks[block_index].fetch_or(mask, Ordering::Acquire);
                        (prev_block | mask) == prev_block
                    },
                    || (),
                    |_, _| (),
                    ParkToken(0),
                    None,
                )
            } {
                ParkResult::Unparked(UnparkToken(unparked_index)) if unparked_index == index => (),
                ParkResult::Unparked(_) => continue,
                ParkResult::Invalid => return, // lock acquired if invalid
                ParkResult::TimedOut => {
                    panic!("Unexpected ParkResult: it's TimedOut but timeout wasn't set")
                }
            }

            let prev_block = self.blocks[block_index].fetch_or(mask, Ordering::Acquire);
            if (prev_block | mask) != prev_block {
                return;
            }
        }
    }
}

impl<const BLOCKS: usize> Groupex for RawGroupex<BLOCKS> {
    fn new() -> Self {
        const { assert!(BLOCKS > 0, "RawGroupex must have more blocks than 0") };
        RawGroupex {
            blocks: [BLOCK_INIT; BLOCKS],
        }
    }

    #[inline]
    fn elements(&self) -> usize {
        BLOCKS * BLOCK_SIZE
    }

    #[inline]
    fn lock(&self, index: usize) {
        self.validate_index(index);
        let mask = get_mask::<BLOCK_SIZE>(index) as u32;
        let prev_block = self.blocks[index / BLOCK_SIZE].fetch_or(mask, Ordering::Acquire);
        if (prev_block | mask) == prev_block {
            self.lock_slow(index, mask);
        }
    }

    #[inline]
    fn try_lock(&self, index: usize) -> bool {
        self.validate_index(index);
        let block_index = index / BLOCK_SIZE;
        let mask = get_mask::<BLOCK_SIZE>(index) as u32;
        let prev_block = self.blocks[block_index].fetch_or(mask, Ordering::Acquire);

        (prev_block | mask) != prev_block
    }

    #[inline]
    fn unlock(&self, index: usize) {
        self.validate_index(index);
        let mask = get_mask::<BLOCK_SIZE>(index) as u32;
        self.blocks[index / BLOCK_SIZE].fetch_and(!mask, Ordering::Release);

        unsafe {
            unpark_all(
                &self.blocks[index / BLOCK_SIZE] as *const _ as usize,
                UnparkToken(index),
            );
        }
    }

    #[inline]
    fn is_locked(&self, index: usize) -> bool {
        self.validate_index(index);
        let block_index = index / BLOCK_SIZE;
        let mask = get_mask::<BLOCK_SIZE>(index) as u32;
        let block = self.blocks[block_index].load(Ordering::Relaxed);

        (block & mask) != 0
    }
}

impl<const BLOCKS: usize> Default for RawGroupex<BLOCKS> {
    fn default() -> Self {
        Self::new()
    }
}
