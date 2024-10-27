use std::{
    hint,
    mem::size_of,
    sync::atomic::{AtomicUsize, Ordering},
};

use parking_lot_core::{park, unpark_filter, FilterOp, ParkToken, UnparkToken};

use crate::{groupex::get_mask, Groupex};

const SPIN_LIMIT: usize = 5;
const PARK_KEY_SHIFT: usize = 52;
const GROUPEX_SIZE: usize = size_of::<usize>() * 8;

// set:    flags | mask
// unset:  flags & !mask

#[derive(Default)]
pub struct RawGroupex {
    flags: AtomicUsize,
}

impl RawGroupex {
    #[cold]
    fn lock_slow(&self, index: usize, mask: usize) {
        for spin_cnt in 0..SPIN_LIMIT {
            let prev_block = self.flags.fetch_or(mask, Ordering::Acquire);
            if (prev_block | mask) != prev_block {
                return;
            }
            for _ in 0..(1 << spin_cnt) {
                hint::spin_loop();
            }
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

}

impl Groupex for RawGroupex {
    fn new() -> Self {
        RawGroupex {
            flags: AtomicUsize::new(0),
        }
    }

    fn elements(&self) -> usize {
        GROUPEX_SIZE
    }

    #[inline]
    fn lock(&self, index: usize) {
        let mask = get_mask::<GROUPEX_SIZE>(index);
        let prev_flags = self.flags.fetch_or(mask, Ordering::Acquire);
        if (prev_flags | mask) == prev_flags {
            self.lock_slow(index, mask);
        }
    }

    #[inline]
    fn try_lock(&self, index: usize) -> bool {
        let mask = get_mask::<GROUPEX_SIZE>(index);
        let prev_flags = self.flags.fetch_or(mask, Ordering::Acquire);

        (prev_flags | mask) != prev_flags
    }

    #[inline]
    fn unlock(&self, index: usize) {
        let mask = get_mask::<GROUPEX_SIZE>(index);
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
    fn is_locked(&self, index: usize) -> bool {
        let mask = get_mask::<GROUPEX_SIZE>(index);
        let flags = self.flags.load(Ordering::Relaxed);

        (flags & mask) != 0
    }
}
