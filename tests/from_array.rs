use orx_concurrent_iter::*;

#[test]
fn con_iter() {
    let values = ['a', 'b', 'c'];

    let con_iter = values.con_iter();
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn into_con_iter() {
    let values = ['a', 'b', 'c'];

    let con_iter = values.into_con_iter();
    assert_eq!(con_iter.next(), Some('a'));
    assert_eq!(con_iter.next(), Some('b'));
    assert_eq!(con_iter.next(), Some('c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values.into_exact_con_iter();
    assert_eq!(con_iter.next(), Some('a'));
    assert_eq!(con_iter.next(), Some('b'));
    assert_eq!(con_iter.next(), Some('c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values.into_iter().take(2).into_con_iter();
    assert_eq!(con_iter.next(), Some('a'));
    assert_eq!(con_iter.next(), Some('b'));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn exact_len() {
    let values = ['a', 'b', 'c'];
    assert_eq!(3, values.exact_len())
}
