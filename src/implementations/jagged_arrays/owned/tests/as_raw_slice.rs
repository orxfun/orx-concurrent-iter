use crate::implementations::jagged_arrays::{
    AsRawSlice, RawSlice, RawVec, as_raw_slice::AsOwningSlice,
};
use alloc::vec;

fn assert_slicing<S>(slice: &S)
where
    S: AsRawSlice<i32>,
{
    let expected = [20, 30, 40];

    let raw = slice.raw_slice(1, expected.len());
    assert_eq!(raw.length(), expected.len());
    assert_eq!(raw.ptr(), unsafe { slice.ptr().add(1) });
    assert_eq!(
        unsafe { core::slice::from_raw_parts(raw.ptr(), raw.length()) },
        &expected
    );

    let raw = unsafe { slice.raw_slice_unchecked(1, expected.len()) };
    assert_eq!(raw.length(), expected.len());
    assert_eq!(raw.ptr(), unsafe { slice.ptr().add(1) });
    assert_eq!(
        unsafe { core::slice::from_raw_parts(raw.ptr(), raw.length()) },
        &expected
    );
}

fn assert_out_of_bounds_panics<S>(slice: &S)
where
    S: AsRawSlice<i32>,
{
    for (begin, len) in [(4, 2), (5, 1), (6, 0)] {
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| slice.raw_slice(begin, len)));
        assert!(result.is_err());
    }
}

#[test]
fn raw_slice_implementations() {
    let values = vec![10, 20, 30, 40, 50];

    let slice = values.as_slice();
    assert_slicing(&slice);
    assert_out_of_bounds_panics(&slice);

    assert_slicing(&values);
    assert_out_of_bounds_panics(&values);

    let raw_vec = unsafe { RawVec::new_from_vec(values.clone()) };
    assert_slicing(&raw_vec);
    assert_out_of_bounds_panics(&raw_vec);
    unsafe { raw_vec.drop_allocation() };

    let raw_slice = RawSlice::from(values.as_slice());
    assert_slicing(&raw_slice);
    assert_out_of_bounds_panics(&raw_slice);
}
