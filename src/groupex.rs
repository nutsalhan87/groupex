pub trait Groupex {
    fn new() -> Self;
    fn elements(&self) -> usize;
    fn lock(&self, index: usize);
    fn try_lock(&self, index: usize) -> bool;
    fn unlock(&self, index: usize);
    fn is_locked(&self, index: usize) -> bool;
}

#[inline]
pub(crate) fn get_mask<const BLOCK_SIZE: usize>(index: usize) -> usize {
    const { assert!(BLOCK_SIZE != 0, "Block size must be grater than 0") };
    1 << (index % BLOCK_SIZE)
}