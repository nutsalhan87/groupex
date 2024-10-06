mod guard;

pub use guard::SizedGroupexParkingLotGuard;
use parking_lot_core::{park, unpark_filter, FilterOp, ParkToken, UnparkToken};

use std::{
    hint,
    mem::size_of,
    sync::atomic::{AtomicU32, Ordering},
};

const SPIN_LIMIT: usize = 5;
const PARK_KEY_SHIFT: u32 = 52;
const BLOCK_SIZE: usize = size_of::<AtomicU32>() * 8;
const BLOCK_INIT: AtomicU32 = AtomicU32::new(0);
const INDEX_MASKS: [u32; BLOCK_SIZE] = {
    let mut index_masks = [0; BLOCK_SIZE];
    index_masks[0] = 1;
    let mut i = 1;
    while i < BLOCK_SIZE {
        index_masks[i] = index_masks[i - 1] << 1;
        i += 1;
    }
    index_masks
};

#[inline]
fn get_mask(index: usize, blocks: usize) -> u32 {
    if index >= BLOCK_SIZE * blocks {
        panic!(
            "Index must be in [0; {}] but it is {}",
            (BLOCK_SIZE * blocks) - 1,
            index
        );
    }

    INDEX_MASKS[index % BLOCK_SIZE]
}

pub struct RawSizedGroupexParkingLot<const BLOCKS: usize> {
    blocks: [AtomicU32; BLOCKS],
}

impl<const BLOCKS: usize> RawSizedGroupexParkingLot<BLOCKS> {
    #[inline]
    pub(crate) fn elements(&self) -> usize {
        BLOCKS * BLOCK_SIZE
    }

    pub fn new() -> Self {
        const { assert!(BLOCKS > 0, "RawGroupex must have more blocks than 0") };
        RawSizedGroupexParkingLot {
            blocks: [BLOCK_INIT; BLOCKS],
        }
    }

    #[inline]
    pub fn lock(&self, index: usize) {
        let block_index = index / BLOCK_SIZE;
        let mask = get_mask(index, BLOCKS);
        let prev_block = self.blocks[block_index].fetch_or(mask, Ordering::Acquire);
        if (prev_block | mask) == prev_block {
            self.lock_slow(block_index, mask);
        }
    }

    #[cold]
    fn lock_slow(&self, block_index: usize, mask: u32) {
        let mut spin_cnt = 0;
        while spin_cnt < SPIN_LIMIT {
            let prev_block = self.blocks[block_index].fetch_or(mask, Ordering::Acquire);
            if (prev_block | mask) != prev_block {
                return;
            }
            for _ in 0..(1 << spin_cnt) {
                hint::spin_loop();
            }
            spin_cnt += 1;
        }

        // lock acquired if false
        let validate_not_locked = || {
            let prev_block = self.blocks[block_index].fetch_or(mask, Ordering::Acquire);
            (prev_block | mask) == prev_block
        };
        loop {
            match unsafe {
                park(
                    (self as *const _ as usize).wrapping_add(block_index << PARK_KEY_SHIFT),
                    validate_not_locked,
                    || (),
                    |_, _| (),
                    ParkToken(self as *const _ as usize),
                    None,
                )
            } {
                parking_lot_core::ParkResult::Unparked(_) => (),
                parking_lot_core::ParkResult::Invalid => return, // lock acquired if invalid
                parking_lot_core::ParkResult::TimedOut => {
                    panic!("Unexpected ParkResult: it's TimedOut but timeout wasn't set")
                }
            }

            if let false = validate_not_locked() {
                return;
            }
        }
    }

    #[inline]
    pub fn try_lock(&self, index: usize) -> bool {
        let block_index = index / BLOCK_SIZE;
        let mask = get_mask(index, BLOCKS);
        let prev_block = self.blocks[block_index].fetch_or(mask, Ordering::Acquire);

        (prev_block | mask) != prev_block
    }

    #[inline]
    pub fn unlock(&self, index: usize) {
        let block_index = index / BLOCK_SIZE;
        let mask = get_mask(index, BLOCKS);
        self.blocks[block_index].fetch_and(!mask, Ordering::Release);

        let mut unparked = false;
        unsafe {
            unpark_filter(
                (self as *const _ as usize).wrapping_add(block_index.wrapping_shl(PARK_KEY_SHIFT)),
                |park_token| {
                    if park_token.0 == self as *const _ as usize {
                        unparked = true;
                        FilterOp::Unpark
                    } else if unparked {
                        FilterOp::Stop
                    } else {
                        FilterOp::Skip
                    }
                },
                |_| UnparkToken(0),
            );
        }
    }

    #[inline]
    pub fn is_locked(&self, index: usize) -> bool {
        let block_index = index / BLOCK_SIZE;
        let mask = get_mask(index, BLOCKS);
        let block = self.blocks[block_index].load(Ordering::Relaxed);

        (block & mask) != 0
    }
}

impl<const BLOCKS: usize> Default for RawSizedGroupexParkingLot<BLOCKS> {
    fn default() -> Self {
        Self::new()
    }
}
