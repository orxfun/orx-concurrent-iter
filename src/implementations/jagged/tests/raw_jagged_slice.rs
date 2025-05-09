use crate::implementations::jagged::raw_jagged_slice::RawJaggedSlice;

#[test]
fn default_raw_jagged_slice() {
    let empty_slice = RawJaggedSlice::<String>::default();
    assert_eq!(empty_slice.num_slices(), 0);
    assert_eq!(empty_slice.get_slice(0), None);
    assert_eq!(empty_slice.get_slice(1), None);
}

// #[test]
// fn
