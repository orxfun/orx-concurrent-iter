use crate::{ChunkPuller, ConcurrentIter, IterIntoConcurrentIter};
use core::ops::Range;
use std::vec::Vec;

#[test]
fn enumerate_item() {
    let range: Range<usize> = 2..6;
    let seq_iter = range.into_iter().filter(|x| x < &3333);
    let iter = seq_iter.iter_into_con_iter().enumerate();
    let mut j = 0;
    while let Some((i, x)) = iter.next() {
        assert_eq!((i, x), (j, j + 2));
        j += 1;
    }

    let range: Range<usize> = 2..6;
    let seq_iter = range.into_iter().filter(|x| x < &3333);
    let iter = seq_iter.iter_into_con_iter().enumerate();
    let mut j = 0;
    while let Some((i2, (i, x))) = iter.next_with_idx() {
        assert_eq!((i, x), (j, j + 2));
        assert_eq!(i, i2);
        j += 1;
    }
}

#[test]
fn enumerate_item_puller() {
    let range: Range<usize> = 2..6;
    let seq_iter = range.into_iter().filter(|x| x < &3333);
    let iter = seq_iter.iter_into_con_iter().enumerate();
    let puller = iter.item_puller();
    let collected: Vec<_> = puller.collect();
    assert_eq!(collected, [0, 1, 2, 3].map(|x| (x, 2 + x)));

    let range: Range<usize> = 2..6;
    let seq_iter = range.into_iter().filter(|x| x < &3333);
    let iter = seq_iter.iter_into_con_iter().enumerate();
    let puller = iter.item_puller_with_idx();
    let collected: Vec<_> = puller.collect();
    assert_eq!(collected, [0, 1, 2, 3].map(|x| (x, (x, 2 + x))));
}

#[test]
fn enumerate_chunk_puller() {
    let range: Range<usize> = 2..6;
    let seq_iter = range.into_iter().filter(|x| x < &3333);
    let iter = seq_iter.iter_into_con_iter().enumerate();

    let mut j = 0;

    let mut puller = iter.chunk_puller(2);
    while let Some(chunk) = puller.pull() {
        assert_eq!(chunk.len(), 2);
        for (i, x) in chunk {
            assert_eq!((i, x), (j, j + 2));
            j += 1;
        }
    }
}

#[test]
fn copied() {
    let vec: Vec<_> = [2, 3, 4, 5].into_iter().collect();
    let seq_iter = vec.iter().filter(|x| **x < 3333);
    let iter = seq_iter.iter_into_con_iter().copied();
    let values: Vec<_> = iter.item_puller().collect();
    assert_eq!(values, vec);
}

#[test]
fn cloned() {
    let vec: Vec<_> = [2, 3, 4, 5].into_iter().collect();
    let seq_iter = vec.iter().filter(|x| **x < 3333);
    let iter = seq_iter.iter_into_con_iter().cloned();
    let values: Vec<_> = iter.item_puller().collect();
    assert_eq!(values, vec);
}
