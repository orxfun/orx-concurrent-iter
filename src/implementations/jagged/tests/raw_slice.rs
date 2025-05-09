use crate::implementations::jagged::raw_slice::RawSlice;

#[test]
fn raw_slice_default() {
    let raw_slice = RawSlice::<String>::default();

    assert_eq!(raw_slice.len(), 0);

    assert_eq!(raw_slice.slice_from(0), None);
    assert_eq!(raw_slice.slice_from(1), None);
}

#[test]
fn raw_slice_from_slice() {
    let vec: Vec<_> = [1, 2, 3, 4, 5].map(|x| x.to_string()).into_iter().collect();
    let raw_slice = RawSlice::from(vec.as_slice());

    assert_eq!(raw_slice.len(), vec.len());

    for i in 0..vec.len() {
        assert_eq!(raw_slice.slice_from(i), Some(&vec[i..]));
    }

    assert_eq!(raw_slice.slice_from(vec.len()), None);
    assert_eq!(raw_slice.slice_from(vec.len() + 1), None);
}
