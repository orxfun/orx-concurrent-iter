use orx_concurrent_iter::*;

#[test]
fn new() {
    let values = ['a', 'b', 'c'];

    let con_iter = ConIterOfIterX::new(values.iter());

    let mut i = 0;
    while let Some(x) = con_iter.next() {
        assert_eq!(x, &values[i]);
        i += 1;
    }
    assert_eq!(i, values.len());
}

#[test]
fn from() {
    let values = ['a', 'b', 'c'];

    let con_iter: ConIterOfIterX<_, _> = values.iter().into();

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
    let con_iter: ConIterOfIterX<_, _> = values.iter().into();

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfIterX { initial_len: 3, taken: 0, remaining: 3 }"
    );

    assert_eq!(con_iter.next(), Some(&'a'));

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfIterX { initial_len: 3, taken: 1, remaining: 2 }"
    );

    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfIterX { initial_len: 3, taken: 3, remaining: 0 }"
    );

    assert_eq!(con_iter.next(), None);

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfIterX { initial_len: 3, taken: 3, remaining: 0 }"
    );

    assert_eq!(con_iter.next(), None);

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfIterX { initial_len: 3, taken: 3, remaining: 0 }"
    );
}
