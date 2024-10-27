//! # Groupex
//! 
//! Syncronization primitive that allows acquire lock by index.
//! 
//! ## [`RawGroupex`]
//! 
//! `RawGroupex` is base on which you can build collections with concurrent access. 
//! It implements basic functions for acquiring and releasing a lock, such as [`lock`](RawGroupex::lock), 
//! [`try_lock`](RawGroupex::try_lock) and [`unlock`](RawGroupex::unlock).
//! These functions recieve index on which lock must be acquired/released.
//! 
//! Example:
//! 
//! ```
//! # use groupex::RawGroupex;
//! # use std::{thread, sync::Arc};
//! let groupex = Arc::new(RawGroupex::<1>::new());
//! thread::scope(|scope| {
//!     for i in 0..10 {
//!         let groupex = &groupex;
//!         scope.spawn(move || {
//!             groupex.lock(i % 2);
//!             // critical section
//!             groupex.unlock(i % 2);
//!         });
//!     }
//! });
//! ```
//! 
//! If lock already acquired by other thread, current thread will go to sleep until the lock is released.
//! `RawGroupex` uses [parking_lot_core](https://docs.rs/parking_lot_core/latest/parking_lot_core/) internally for parking and unparking threads.
//! 
//! These functions will panic if index out of range. Range depends on generic parameter `BLOCKS`.
//! One block contains 32 slots so the range of possible indexes is [0; BLOCKS * 32 - 1]. For example, this code will panic:
//! 
//! ```should_panic
//! # use groupex::RawGroupex;
//! let groupex = RawGroupex::<8>::new();
//! groupex.lock(8 * 32);
//! ```
//! 
//! `RawGroupex` also has auxiliary functions [`is_locked`](RawGroupex::is_locked) and [`elements`](RawGroupex::elements). 
//! The first one will be useful to check if index currently acquired.
//! The second one will return number of indexes and will useful when creating other data structures on top of `RawGroupex`.
//! 
//! ## [`GroupexMap`] and [`GroupexVec`]
//! 
//! This crate provides `GroupexMap` and `GroupexVec` structs - hash table and array.
//! 
//! ### `GroupexMap`
//! 
//! `GroupexMap` is hash table built on top of `HashMap` and `RawGroupex`. 
//! 
//! Its cells can be locked by keys of any type that implement traits [`Eq`] and [`Hash`]. 
//! Computed hash will be simply divided by [`elements()`](RawGroupex::elements)'s result and remainder of the division will be used as index for locking.
//! Thus different keys can be resolved to the same index. So **it's dangerous to lock another cell when one lock already acquired in the thread**.
//! 
//! Take into account that using just `HashMap<_, Mutex<_>>` is faster than `GroupexMap`.
//! The second one is your choice only if you need more space-efficient solution.
//! 
//! ### `GroupexVec`
//! 
//! `GroupexVec` works similar to the previous one but includes [`Vec`] and its cells can be locked just by index.
//! It's also **dangerous to lock another cell when one lock already acquired in the thread** because different indexes can be collided due to size of the `RawGroupex`.


mod groupex_guard;
mod groupex_map;
mod groupex_vec;
mod raw_groupex;

pub use groupex_guard::GroupexGuard;
pub use groupex_map::GroupexMap;
pub use groupex_vec::GroupexVec;
pub use raw_groupex::RawGroupex;
