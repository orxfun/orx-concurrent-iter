use crate::AtomicCounter;

pub fn fmt_iter(
    f: &mut std::fmt::Formatter<'_>,
    struct_name: &str,
    initial_len: usize,
    counter: &AtomicCounter,
) -> std::fmt::Result {
    let num_taken = counter.current().min(initial_len);
    let remaining = initial_len - num_taken;
    f.debug_struct(struct_name)
        .field("initial_len", &initial_len)
        .field("taken", &num_taken)
        .field("remaining", &remaining)
        .finish()
}
