use orx_concurrent_iter::*;

#[test]
fn con_iter() {
    let values = ['a', 'b', 'c'];
    let slice = values.as_slice();

    let con_iter = slice.con_iter();
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = slice.into_con_iter();
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = slice.into_exact_con_iter();
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn exact_len() {
    let values = vec!['a', 'b', 'c'];
    let slice = values.as_slice();
    assert_eq!(3, slice.exact_len())
}

#[test]
fn len() {
    let values = vec!['a', 'b', 'c', 'd'];
    let slice = values.as_slice();

    let iter = slice.con_iter();
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
