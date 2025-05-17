use crate::concurrent_iterable::ConcurrentIterable;
use crate::{ChunkPuller, ConcurrentIter};
use alloc::vec::Vec;
use core::ops::Range;

#[test]
fn enumerate_item() {
    let range: Range<usize> = 2..6;
    let iter = range.con_iter().enumerate();
    let mut j = 0;
    while let Some((i, x)) = iter.next() {
        assert_eq!((i, x), (j, j + 2));
        j += 1;
    }

    let range: Range<usize> = 2..6;
    let iter = range.con_iter().enumerate();
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
    let iter = range.con_iter().enumerate();
    let puller = iter.item_puller();
    let collected: Vec<_> = puller.collect();
    assert_eq!(collected, [0, 1, 2, 3].map(|x| (x, 2 + x)));

    let range: Range<usize> = 2..6;
    let iter = range.con_iter().enumerate();
    let puller = iter.item_puller_with_idx();
    let collected: Vec<_> = puller.collect();
    assert_eq!(collected, [0, 1, 2, 3].map(|x| (x, (x, 2 + x))));
}

#[test]
fn enumerate_chunk_puller() {
    let range: Range<usize> = 2..6;

    let mut j = 0;

    let iter = range.con_iter().enumerate();
    let mut puller = iter.chunk_puller(2);
    while let Some(chunk) = puller.pull() {
        assert_eq!(chunk.len(), 2);
        for (i, x) in chunk {
            assert_eq!((i, x), (j, j + 2));
            j += 1;
        }
    }
}
