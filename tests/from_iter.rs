use orx_concurrent_iter::*;

#[test]
fn con_iter() {
    let values = ['a', 'b', 'c'];

    let con_iter = values.iter().into_con_iter();
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values.iter().take(2).into_con_iter();
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values.iter().skip(1).into_con_iter();
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values
        .iter()
        .filter(|x| **x != 'a')
        .map(|x| x.to_string())
        .into_con_iter();
    assert_eq!(con_iter.next(), Some(String::from('b')));
    assert_eq!(con_iter.next(), Some(String::from('c')));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn len() {
    let values = ['a', 'b', 'c', 'd'];
    let iter = values.iter();

    assert_eq!(iter.try_get_exact_len(), None);

    let con_iter = iter.into_con_iter();
    assert_eq!(con_iter.try_get_len(), None);

    _ = con_iter.next();
    assert_eq!(con_iter.try_get_len(), None);

    _ = con_iter.next_chunk(2);
    assert_eq!(con_iter.try_get_len(), None);

    _ = con_iter.next_chunk(2);
    assert_eq!(con_iter.try_get_len(), None);

    _ = con_iter.next();
    assert_eq!(con_iter.try_get_len(), None);

    _ = con_iter.next();
    assert_eq!(con_iter.try_get_len(), None);
}
