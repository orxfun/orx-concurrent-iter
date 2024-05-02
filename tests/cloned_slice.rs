use orx_concurrent_iter::*;

#[test]
fn cloned_slice() {
    let values = ['a', 'b', 'c'].map(String::from);
    let slice = values.as_slice();

    let con_iter = slice.con_iter().cloned();
    assert_eq!(con_iter.next(), Some("a".to_string()));
    assert_eq!(con_iter.next(), Some("b".to_string()));
    assert_eq!(con_iter.next(), Some("c".to_string()));
    assert_eq!(con_iter.next(), None);

    let con_iter = slice.into_con_iter().cloned();
    assert_eq!(con_iter.next(), Some("a".to_string()));
    assert_eq!(con_iter.next(), Some("b".to_string()));
    assert_eq!(con_iter.next(), Some("c".to_string()));
    assert_eq!(con_iter.next(), None);

    let con_iter = slice.into_exact_con_iter().cloned();
    assert_eq!(con_iter.next(), Some("a".to_string()));
    assert_eq!(con_iter.next(), Some("b".to_string()));
    assert_eq!(con_iter.next(), Some("c".to_string()));
    assert_eq!(con_iter.next(), None);
}
