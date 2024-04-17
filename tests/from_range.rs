use orx_concurrent_iter::{
    ConIterOfIter, ConIterOfRange, ConcurrentIter, ConcurrentIterable, IntoConcurrentIter,
    IntoExactSizeConcurrentIter,
};

#[test]
fn con_iter() {
    let values = 42..45;

    let con_iter: ConIterOfRange<_> = values.con_iter();
    assert_eq!(con_iter.next(), Some(42));
    assert_eq!(con_iter.next(), Some(43));
    assert_eq!(con_iter.next(), Some(44));
    assert_eq!(con_iter.next(), None);

    let con_iter: ConIterOfRange<_> = values.clone().into_con_iter();
    assert_eq!(con_iter.next(), Some(42));
    assert_eq!(con_iter.next(), Some(43));
    assert_eq!(con_iter.next(), Some(44));
    assert_eq!(con_iter.next(), None);

    let con_iter: ConIterOfRange<_> = values.into_exact_con_iter();
    assert_eq!(con_iter.next(), Some(42));
    assert_eq!(con_iter.next(), Some(43));
    assert_eq!(con_iter.next(), Some(44));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn into_con_iter() {
    use orx_concurrent_iter::IterIntoConcurrentIter;

    let values = 42..45;

    let con_iter: ConIterOfIter<_, _> = values.into_iter().take(2).into_con_iter();
    assert_eq!(con_iter.next(), Some(42));
    assert_eq!(con_iter.next(), Some(43));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn exact_len() {
    let values = 42..45;
    assert_eq!(3, values.exact_len());
}
