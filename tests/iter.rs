use orx_concurrent_iter::{implementations::ConIterOfIter, *};

#[test]
fn new() {
    let values = ['a', 'b', 'c'];

    let con_iter = values.iter().iter_into_con_iter();

    let mut i = 0;
    while let Some(x) = con_iter.next() {
        assert_eq!(x, &values[i]);
        i += 1;
    }
    assert_eq!(i, values.len());
}

#[test]
fn debug() {
    let values = ['a', 'b', 'c'];
    let con_iter: ConIterOfIter<_> = values.iter().iter_into_con_iter();

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfIter { size_hint: (3, Some(3)) }"
    );

    assert_eq!(con_iter.next(), Some(&'a'));

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfIter { size_hint: (2, Some(2)) }"
    );

    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfIter { size_hint: (0, Some(0)) }"
    );

    assert_eq!(con_iter.next(), None);

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfIter { size_hint: (0, Some(0)) }"
    );

    assert_eq!(con_iter.next(), None);

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfIter { size_hint: (0, Some(0)) }"
    );
}
