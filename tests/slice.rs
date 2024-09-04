use orx_concurrent_iter::*;

#[test]
fn new() {
    let values = ['a', 'b', 'c'];
    let slice = values.as_slice();

    let con_iter = ConIterOfSlice::new(slice);
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn from() {
    let values = ['a', 'b', 'c'];
    let slice = values.as_slice();

    let con_iter = ConIterOfSlice::new(slice);
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn debug() {
    let values = ['a', 'b', 'c'];
    let con_iter: ConIterOfSlice<_> = values.as_slice().into();

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfSlice { initial_len: 3, taken: 0, remaining: 3 }"
    );

    assert_eq!(con_iter.next(), Some(&'a'));

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfSlice { initial_len: 3, taken: 1, remaining: 2 }"
    );

    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfSlice { initial_len: 3, taken: 3, remaining: 0 }"
    );

    assert_eq!(con_iter.next(), None);

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfSlice { initial_len: 3, taken: 3, remaining: 0 }"
    );
}

#[test]
fn as_slice() {
    let values = ['a', 'b', 'c'];
    let slice = values.as_slice();
    let vec = slice.to_vec();

    let con_iter: ConIterOfSlice<_> = slice.into();

    assert_eq!(con_iter.next(), Some(&'a'));

    assert_eq!(slice, &vec);
}

#[test]
fn clone() {
    let values = ['a', 'b', 'c'];
    let slice = values.as_slice();

    let con_iter: ConIterOfSlice<_> = slice.into();

    assert_eq!(con_iter.try_get_len(), Some(3));

    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.try_get_len(), Some(2));

    let clone = con_iter.clone();
    assert_eq!(con_iter.try_get_len(), Some(2));
    assert_eq!(clone.try_get_len(), Some(2));

    assert_eq!(clone.next(), Some(&'b'));
    assert_eq!(clone.next(), Some(&'c'));
    assert_eq!(con_iter.try_get_len(), Some(2));
    assert_eq!(clone.try_get_len(), Some(0));

    assert_eq!(clone.next(), None);
    assert_eq!(con_iter.try_get_len(), Some(2));
    assert_eq!(clone.try_get_len(), Some(0));
}
