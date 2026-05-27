use crate::concurrent_collection::ConcurrentCollection;
use crate::concurrent_iter::ConcurrentIter;
use std::hint::black_box;
use std::iter::Iterator;

#[test]
fn enumerated_item_puller_size_hint_empty() {
    let data: Vec<i32> = vec![];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller_with_idx();

    let (lower, upper) = puller.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(0));
}

#[test]
fn enumerated_item_puller_size_hint_non_empty() {
    let data: Vec<i32> = (0..100).collect();
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller_with_idx();

    let (lower, upper) = puller.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(100));
}

#[test]
fn enumerated_item_puller_size_hint_small() {
    let data = vec![10, 20, 30];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller_with_idx();

    let (lower, upper) = puller.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(3));
}

#[test]
fn enumerated_item_puller_fold_with_indices() {
    let data = vec![10, 20, 30, 40];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller_with_idx();

    let result = puller.fold(Vec::new(), |mut acc, (idx, &value)| {
        acc.push((idx, value));
        acc
    });

    assert_eq!(result.len(), 4);
    // Check that we have indices and values correctly paired
    for (idx, (res_idx, _)) in result.iter().enumerate() {
        assert_eq!(idx, *res_idx);
    }
}

#[test]
fn enumerated_item_puller_fold_sum_values_with_indices() {
    let data = vec![1, 2, 3, 4, 5];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller_with_idx();

    let sum = puller.fold(0, |acc, (_idx, &value)| acc + value);
    assert_eq!(sum, 15); // 1+2+3+4+5 = 15
}

#[test]
fn enumerated_item_puller_fold_sum_indices() {
    let data = vec![100, 200, 300];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller_with_idx();

    let index_sum = puller.fold(0, |acc, (idx, _)| acc + idx);
    assert_eq!(index_sum, 3); // 0+1+2 = 3
}

#[test]
fn enumerated_item_puller_fold_collect_pairs() {
    let data = vec!["a", "b", "c"];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller_with_idx();

    let pairs = puller.fold(Vec::new(), |mut acc, (idx, &value)| {
        acc.push((idx, value));
        acc
    });

    assert_eq!(pairs, vec![(0, "a"), (1, "b"), (2, "c")]);
}

#[test]
fn enumerated_item_puller_fold_empty() {
    let data: Vec<i32> = vec![];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller_with_idx();

    let result = puller.fold(0, |acc, (_idx, &value)| acc + value);
    assert_eq!(result, 0);
}

#[test]
fn enumerated_item_puller_fold_with_init() {
    let data = vec![10, 20, 30];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller_with_idx();

    let result = puller.fold(100, |acc, (_idx, &value)| acc + value);
    assert_eq!(result, 160); // 100 + 10 + 20 + 30 = 160
}

#[test]
fn enumerated_item_puller_count_empty() {
    let data: Vec<i32> = vec![];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller_with_idx();

    let count = puller.count();
    assert_eq!(count, 0);
}

#[test]
fn enumerated_item_puller_count_single() {
    let data = vec![42];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller_with_idx();

    let count = puller.count();
    assert_eq!(count, 1);
}

#[test]
fn enumerated_item_puller_count_multiple() {
    let data: Vec<i32> = (0..50).collect();
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller_with_idx();

    let count = puller.count();
    assert_eq!(count, 50);
}

#[test]
fn enumerated_item_puller_count_with_strings() {
    let data = vec!["alpha", "beta", "gamma", "delta"];
    let con_iter = data.con_iter();
    let puller = con_iter.item_puller_with_idx();

    let count = puller.count();
    assert_eq!(count, 4);
}

// ============================================================================
// Comparison Tests: ItemPuller vs EnumeratedItemPuller
// ============================================================================

#[test]
fn item_and_enumerated_count_same() {
    let data: Vec<i32> = (0..25).collect();

    let con_iter1 = data.con_iter();
    let puller1 = con_iter1.item_puller();
    let count1 = puller1.count();

    let con_iter2 = data.con_iter();
    let puller2 = con_iter2.item_puller_with_idx();
    let count2 = puller2.count();

    assert_eq!(count1, count2);
    assert_eq!(count1, 25);
}

#[test]
fn item_and_enumerated_sum_values_same() {
    let data: Vec<i32> = (1..=20).collect();

    let con_iter1 = data.con_iter();
    let puller1 = con_iter1.item_puller();
    let sum1 = puller1.fold(0, |acc, &x| black_box(acc + x));

    let con_iter2 = data.con_iter();
    let puller2 = con_iter2.item_puller_with_idx();
    let sum2 = puller2.fold(0, |acc, (_idx, &x)| black_box(acc + x));

    assert_eq!(sum1, sum2);
    assert_eq!(sum1, 210); // 1+2+...+20 = 20*21/2 = 210
}
