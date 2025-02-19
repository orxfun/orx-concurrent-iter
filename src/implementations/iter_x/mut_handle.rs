use core::sync::atomic::{AtomicU8, Ordering};

pub(super) type State = AtomicU8;

pub(super) const AVAILABLE: u8 = 0;
pub(super) const IS_MUTATING: u8 = 1;
pub(super) const COMPLETED: u8 = 2;

pub(super) struct MutHandle<'a> {
    state: &'a State,
}

impl<'a> MutHandle<'a> {
    pub(super) fn get_handle(state: &'a State) -> Option<Self> {
        loop {
            match state.compare_exchange(
                AVAILABLE,
                IS_MUTATING,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Some(Self { state }),
                Err(COMPLETED) => return None,
                _ => {}
            }
        }
    }
}
