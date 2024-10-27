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
    fn lock_slow(&self, block_index: usize, mask: u32) {
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
            let prev_block = self.blocks[block_index].fetch_or(mask, Ordering::Acquire);
            if (prev_block | mask) != prev_block {
                return;
            }
            unsafe {
                libc::syscall(
                    libc::SYS_futex,
                    &self.blocks[block_index],
                    libc::FUTEX_WAIT | libc::FUTEX_PRIVATE_FLAG,
                    prev_block,
                    core::ptr::null::<libc::timespec>(),
                    core::ptr::null::<u32>(),
                    0u32
                );
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
        let block_index = index / BLOCK_SIZE;
        let mask = get_mask::<BLOCK_SIZE>(index) as u32;
        let prev_block = self.blocks[block_index].fetch_or(mask, Ordering::Acquire);
        if (prev_block | mask) == prev_block {
            self.lock_slow(block_index, mask);
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
        let block_index = index / BLOCK_SIZE;
        let mask = get_mask::<BLOCK_SIZE>(index) as u32;
        self.blocks[block_index].fetch_and(!mask, Ordering::Release);
        unsafe {
            libc::syscall(
                libc::SYS_futex,
                &self.blocks[block_index],
                libc::FUTEX_WAKE | libc::FUTEX_PRIVATE_FLAG,
                libc::INT_MAX,
                core::ptr::null::<libc::timespec>(),
                core::ptr::null::<u32>(),
                0u32
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
