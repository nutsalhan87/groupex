use std::{
    hint,
    mem::size_of,
    sync::atomic::{AtomicUsize, Ordering},
};

use parking_lot_core::{park, unpark_filter, FilterOp, ParkToken, UnparkToken};

const SPIN_LIMIT: usize = 5;
const PARK_KEY_SHIFT: usize = 52;
pub(crate) const GROUPEX_SIZE: usize = size_of::<usize>() * 8;
const INDEX_MASKS: [usize; GROUPEX_SIZE] = {
    let mut index_masks = [0; GROUPEX_SIZE];
    index_masks[0] = 1;
    let mut i = 1;
    while i < GROUPEX_SIZE {
        index_masks[i] = index_masks[i - 1] << 1;
        i += 1;
    }
    index_masks
};
// set:    flags | mask
// unset:  flags & !mask

#[derive(Default)]
pub struct RawGroupex {
    flags: AtomicUsize,
}

impl RawGroupex {
    pub fn new() -> Self {
        RawGroupex {
            flags: AtomicUsize::new(0),
        }
    }

    #[inline]
    pub fn lock(&self, index: usize) {
        let mask = Self::get_mask(index);
        let mut spin_cnt = 0;

        while spin_cnt < SPIN_LIMIT {
            let prev_flags = self.flags.fetch_or(mask, Ordering::Acquire);
            if (prev_flags | mask) != prev_flags {
                return;
            }
            for _ in 0..(1 << spin_cnt) {
                hint::spin_loop();
            }
            spin_cnt += 1;
        }

        // lock acquired if false
        let validate_not_locked = || {
            let prev_flags = self.flags.fetch_or(mask, Ordering::Acquire);
            (prev_flags | mask) == prev_flags
        };
        loop {
            match unsafe {
                park(
                    (self as *const _ as usize).wrapping_add(index << PARK_KEY_SHIFT),
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
        let mask = Self::get_mask(index);
        let prev_flags = self.flags.fetch_or(mask, Ordering::Acquire);

        (prev_flags | mask) != prev_flags
    }

    #[inline]
    pub fn unlock(&self, index: usize) {
        let mask = Self::get_mask(index);
        self.flags.fetch_and(!mask, Ordering::Release);
        let mut unparked = false;
        unsafe {
            unpark_filter(
                (self as *const _ as usize).wrapping_add(index << PARK_KEY_SHIFT),
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
        let mask = Self::get_mask(index);
        let flags = self.flags.load(Ordering::Relaxed);

        (flags & mask) != 0
    }

    #[inline]
    fn get_mask(index: usize) -> usize {
        if index >= 64 {
            panic!("Index must be in [0; 63] but it is {index}");
        }

        INDEX_MASKS[index]
    }
}
