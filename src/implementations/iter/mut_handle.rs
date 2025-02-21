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
}
