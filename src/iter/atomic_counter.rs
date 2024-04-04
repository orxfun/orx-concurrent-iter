use std::sync::atomic::{AtomicUsize, Ordering};

/// An atomic counter, simply a wrapper around `AtomicUsize` with utility methods useful for atomic iterators.
#[derive(Debug)]
pub struct AtomicCounter {
    current: AtomicUsize,
}

impl AtomicCounter {
    /// Creates a new atomic counter with its value initiated at zero.
    pub fn new() -> Self {
        Self { current: 0.into() }
    }

    /// Fetches and returns the current value of the counter, and adds `len` to it.
    #[inline(always)]
    pub fn fetch_and_add(&self, len: usize) -> usize {
        self.current.fetch_add(len, Ordering::AcqRel)
    }

    /// Fetches and returns the current value of the counter, and adds `1` to it.
    #[inline(always)]
    pub fn fetch_and_increment(&self) -> usize {
        self.current.fetch_add(1, Ordering::AcqRel)
    }

    /// Fetches and returns the current value of the counter.
    #[inline(always)]
    pub fn current(&self) -> usize {
        self.current.load(Ordering::Relaxed)
    }
}

impl Default for AtomicCounter {
    /// Creates a new atomic counter with its value initiated at zero.
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AtomicCounter {
    /// Clones the atomic counter with its current value.
    fn clone(&self) -> Self {
        Self {
            current: self.current.load(Ordering::SeqCst).into(),
        }
    }
}
