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
