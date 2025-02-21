use core::sync::atomic::{AtomicIsize, Ordering};

pub(super) type AtomicState = AtomicIsize;
pub(super) const COMPLETED: isize = isize::MIN;

pub(super) struct MutHandle<'a> {
    state: &'a AtomicState,
    neg_count_before: isize,
    count_after: usize,
}

impl<'a> MutHandle<'a> {
    pub(super) fn get_handle(state: &'a AtomicState, num_to_pull: usize) -> Option<Self> {
        loop {
            let update = state.fetch_update(Ordering::Acquire, Ordering::Relaxed, |count_before| {
                match count_before >= 0 {
                    true => Some(-count_before),
                    false => None,
                }
            });
            match update {
                Ok(count_before) => {
                    return Some(Self {
                        state,
                        neg_count_before: -count_before,
                        count_after: count_before as usize + num_to_pull,
                    })
                }
                Err(COMPLETED) => return None,
                _ => {}
            }
        }
    }

    pub(super) fn set_count_after(&mut self, num_actually_pulled: usize) {
        self.count_after = (-self.neg_count_before) as usize + num_actually_pulled
    }
}

impl<'a> Drop for MutHandle<'a> {
    fn drop(&mut self) {
        match self.state.compare_exchange(
            self.neg_count_before,
            self.count_after as isize,
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
