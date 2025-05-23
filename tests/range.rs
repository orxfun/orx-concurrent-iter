use orx_concurrent_iter::{implementations::ConIterRange, *};

#[test]
fn new() {
    let range = 3..10;
    let con_iter: ConIterRange<_> = range.into_con_iter();

    let mut i = 0;
    while let Some(x) = con_iter.next() {
        assert_eq!(x, 3 + i);
        i += 1;
    }
    assert_eq!(i, 7);
}

#[test]
fn debug() {
    let range = 3..10;
    let con_iter: ConIterRange<_> = range.into_con_iter();

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterRange { initial_len: 7, num_taken: 0, remaining: 7 }"
    );

    assert_eq!(con_iter.next(), Some(3));

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterRange { initial_len: 7, num_taken: 1, remaining: 6 }"
    );

    for _ in 0..6 {
        _ = con_iter.next();
    }

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterRange { initial_len: 7, num_taken: 7, remaining: 0 }"
    );

    _ = con_iter.next();

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterRange { initial_len: 7, num_taken: 7, remaining: 0 }"
    );
}

#[test]
fn clone() {
    let range = 3..6;
    let con_iter: ConIterRange<_> = range.into_con_iter();

    assert_eq!(con_iter.try_get_len(), Some(3));

    assert_eq!(con_iter.next(), Some(3));
    assert_eq!(con_iter.try_get_len(), Some(2));

    let clone = con_iter.clone();
    assert_eq!(con_iter.try_get_len(), Some(2));
    assert_eq!(clone.try_get_len(), Some(2));

    assert_eq!(clone.next(), Some(4));
    assert_eq!(clone.next(), Some(5));
    assert_eq!(con_iter.try_get_len(), Some(2));
    assert_eq!(clone.try_get_len(), Some(0));

    assert_eq!(clone.next(), None);
    assert_eq!(con_iter.try_get_len(), Some(2));
    assert_eq!(clone.try_get_len(), Some(0));
}
