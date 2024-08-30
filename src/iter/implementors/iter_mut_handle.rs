use super::iter_mut_states::*;
use std::sync::atomic::{AtomicU8, Ordering};

pub(crate) struct IterMutHandle<'a> {
    state: &'a AtomicU8,
}

impl<'a> IterMutHandle<'a> {
    pub fn spin_get(state: &'a AtomicU8) -> Option<Self> {
        loop {
            match state.compare_exchange(
                AVAILABLE,
                IS_MUTATING,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Some(Self { state }),
                Err(previous_state) => match previous_state {
                    IS_MUTATING => continue,
                    _/*COMPLETED*/ => return None,
                },
            }
        }
    }
}

impl<'a> Drop for IterMutHandle<'a> {
    fn drop(&mut self) {
        match self.state.compare_exchange(
            IS_MUTATING,
            AVAILABLE,
            Ordering::Release,
            Ordering::Relaxed,
        ) {
            Ok(_) => {}
            Err(state) => {
                assert_eq!(state, COMPLETED)
            }
        }
    }
}
