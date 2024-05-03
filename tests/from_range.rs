use orx_concurrent_iter::{
    ConIterOfIter, ConIterOfRange, ConcurrentIter, ConcurrentIterable, ExactSizeConcurrentIter,
    IntoConcurrentIter, IntoExactSizeConcurrentIter,
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

#[test]
fn len() {
    let values = 42..46;

    let iter = values.con_iter();
    assert_eq!(iter.len(), 4);
    assert_eq!(iter.try_get_len(), Some(4));

    _ = iter.next();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.try_get_len(), Some(3));

    _ = iter.next_chunk(2);
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.try_get_len(), Some(1));

    _ = iter.next();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.try_get_len(), Some(0));

    _ = iter.next();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.try_get_len(), Some(0));
}
