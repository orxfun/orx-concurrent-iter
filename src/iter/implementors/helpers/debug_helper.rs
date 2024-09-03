use std::sync::atomic::{AtomicUsize, Ordering};

pub fn fmt_iter(
    f: &mut std::fmt::Formatter<'_>,
    struct_name: &str,
    initial_len: Option<usize>,
    counter: &AtomicUsize,
) -> std::fmt::Result {
    let num_taken = counter
        .load(Ordering::Relaxed)
        .min(initial_len.unwrap_or(usize::MAX));

    match initial_len {
        None => f
            .debug_struct(struct_name)
            .field("taken", &num_taken)
            .finish(),
        Some(initial_len) => f
            .debug_struct(struct_name)
            .field("initial_len", &initial_len)
            .field("taken", &num_taken)
            .field("remaining", &(initial_len - num_taken))
            .finish(),
    }
}
