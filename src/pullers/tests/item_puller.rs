use crate::concurrent_collection::ConcurrentCollection;
use crate::concurrent_iter::ConcurrentIter;
use std::iter::Iterator;

#[test]
fn item_puller_size_hint_empty() {
    let data: Vec<i32> = vec![];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller();

    let (lower, upper) = puller.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(0));
}

#[test]
fn item_puller_size_hint_non_empty() {
    let data: Vec<i32> = (0..100).collect();
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller();

    let (lower, upper) = puller.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(100));
}

#[test]
fn item_puller_size_hint_small_vec() {
    let data = vec![1, 2, 3, 4, 5];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller();

    let (lower, upper) = puller.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(5));
}

#[test]
fn item_puller_fold_sum() {
    let data: Vec<i32> = (1..=10).collect();
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller();

    let sum = puller.fold(0, |acc, &x| acc + x);
    assert_eq!(sum, 55); // 1+2+3+...+10 = 55
}

#[test]
fn item_puller_fold_product() {
    let data = vec![1, 2, 3, 4, 5];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller();

    let product = puller.fold(1, |acc, &x| acc * x);
    assert_eq!(product, 120); // 5! = 120
}

#[test]
fn item_puller_fold_concatenation() {
    let data: Vec<&str> = vec!["hello", " ", "world"];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller();

    let result = puller.fold(String::new(), |acc, &x| acc + x);
    assert_eq!(result, "hello world");
}

#[test]
fn item_puller_fold_collection() {
    let data = vec![1, 2, 3, 4, 5];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller();

    let collected = puller.fold(Vec::new(), |mut acc, &x| {
        acc.push(x * 2);
        acc
    });
    assert_eq!(collected, vec![2, 4, 6, 8, 10]);
}

#[test]
fn item_puller_fold_empty() {
    let data: Vec<i32> = vec![];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller();

    let sum = puller.fold(0, |acc, &x| acc + x);
    assert_eq!(sum, 0);
}

#[test]
fn item_puller_fold_with_init() {
    let data = vec![1, 2, 3];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller();

    let result = puller.fold(10, |acc, &x| acc + x);
    assert_eq!(result, 16); // 10 + 1 + 2 + 3 = 16
}

#[test]
fn item_puller_count_empty() {
    let data: Vec<i32> = vec![];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller();

    let count = puller.count();
    assert_eq!(count, 0);
}

#[test]
fn item_puller_count_single() {
    let data = vec![42];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller();

    let count = puller.count();
    assert_eq!(count, 1);
}

#[test]
fn item_puller_count_multiple() {
    let data: Vec<i32> = (0..100).collect();
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller();

    let count = puller.count();
    assert_eq!(count, 100);
}

#[test]
fn item_puller_count_strings() {
    let data = vec!["a", "b", "c", "d", "e"];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller();

    let count = puller.count();
    assert_eq!(count, 5);
}
