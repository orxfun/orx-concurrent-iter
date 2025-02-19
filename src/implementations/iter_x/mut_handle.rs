use core::sync::atomic::{AtomicU8, Ordering};

pub(super) type State = AtomicU8;

pub(super) const AVAILABLE: u8 = 0;
pub(super) const IS_MUTATING: u8 = 1;
pub(super) const COMPLETED: u8 = 2;

pub(super) struct MutHandle<'a> {
    state: &'a State,
    final_state: u8,
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

impl<'a> Drop for MutHandle<'a> {
    fn drop(&mut self) {
        self.state
            .compare_exchange(
                IS_MUTATING,
                self.final_state,
                Ordering::Release,
                Ordering::Relaxed,
            )
            .expect("Failed to update the concurrent state after concurrent state mutation");
    }
}
