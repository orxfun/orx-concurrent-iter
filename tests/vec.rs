use orx_concurrent_iter::{implementations::ConIterVec, *};

#[test]
fn new() {
    let values = vec!['a', 'b', 'c'];
    let con_iter: ConIterVec<_> = values.into_con_iter();

    let mut collected = vec![];
    while let Some(x) = con_iter.next() {
        collected.push(x);
    }

    assert_eq!(collected, vec!['a', 'b', 'c']);
}

#[test]
fn debug() {
    let mut values = Vec::with_capacity(4);
    values.extend(['a', 'b', 'c']);
    let con_iter: ConIterVec<_> = values.into_con_iter();

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterVec { initial_len: 3, taken: 0, remaining: 3 }"
    );

    assert_eq!(con_iter.next(), Some('a'));

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterVec { initial_len: 3, taken: 1, remaining: 2 }"
    );

    assert_eq!(con_iter.next(), Some('b'));
    assert_eq!(con_iter.next(), Some('c'));

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterVec { initial_len: 3, taken: 3, remaining: 0 }"
    );

    assert_eq!(con_iter.next(), None);

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterVec { initial_len: 3, taken: 3, remaining: 0 }"
    );
}
