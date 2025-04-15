use core::sync::atomic::{AtomicU8, Ordering};

pub(super) type AtomicState = AtomicU8;

pub(super) const AVAILABLE: u8 = 0;
pub(super) const IS_MUTATING: u8 = 1;
pub(super) const COMPLETED: u8 = 2;

pub(super) struct MutHandle<'a> {
    state: &'a AtomicState,
    final_state: u8,
}

impl<'a> MutHandle<'a> {
    pub(super) fn get_handle(state: &'a AtomicState) -> Option<Self> {
        loop {
            match state.compare_exchange(
                AVAILABLE,
                IS_MUTATING,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    return Some(Self {
                        state,
                        final_state: AVAILABLE,
                    })
                }
                Err(COMPLETED) => return None,
                _ => {}
            }
        }
    }

    pub(super) fn set_target_to_completed(&mut self) {
        self.final_state = COMPLETED;
    }
}

impl Drop for MutHandle<'_> {
    fn drop(&mut self) {
        match self.state.compare_exchange(
            IS_MUTATING,
            self.final_state,
            Ordering::Release,
            Ordering::Relaxed,
        ) {
            Ok(_) => {}
            Err(s) => {
                assert_eq!(
                    s,
                    COMPLETED, // possible due to skip_to_end
                    "Failed to update the concurrent state after concurrent state mutation"
                );
            }
        };
    }
}
