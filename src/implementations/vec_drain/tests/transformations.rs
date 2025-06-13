use crate::{ChunkPuller, ConcurrentDrainableOverSlice, ConcurrentIter};
use alloc::vec::Vec;

#[test]
fn enumerate_item() {
    let mut vec: Vec<_> = (2..6).collect();
    let iter = vec.con_drain(..).enumerate();
    let mut j = 0;
    while let Some((i, x)) = iter.next() {
        assert_eq!((i, x), (j, (j + 2)));
        j += 1;
    }

    let mut vec: Vec<_> = (2..6).collect();
    let iter = vec.con_drain(..).enumerate();
    let mut j = 0;
    while let Some((i2, (i, x))) = iter.next_with_idx() {
        assert_eq!((i, x), (j, (j + 2)));
        assert_eq!(i, i2);
        j += 1;
    }
}

#[test]
fn enumerate_item_puller() {
    let mut vec: Vec<_> = (2..6).collect();
    let iter = vec.con_drain(..).enumerate();
    let puller = iter.item_puller();
    let collected: Vec<_> = puller.collect();
    assert_eq!(collected, [0, 1, 2, 3].map(|x| (x, x + 2)));

    let mut vec: Vec<_> = (2..6).collect();
    let iter = vec.con_drain(..).enumerate();
    let puller = iter.item_puller_with_idx();
    let collected: Vec<_> = puller.collect();
    assert_eq!(collected, [0, 1, 2, 3].map(|x| (x, (x, x + 2))));
}

#[test]
fn enumerate_chunk_puller() {
    let mut vec: Vec<_> = (2..6).collect();
    let iter = vec.con_drain(..).enumerate();

    let mut j = 0;

    let mut puller = iter.chunk_puller(2);
    while let Some(chunk) = puller.pull() {
        assert_eq!(chunk.len(), 2);
        for (i, x) in chunk {
            assert_eq!((i, x), (j, (j + 2)));
            j += 1;
        }
    }
}

#[test]
fn copied() {
    let source: Vec<_> = (2..6).collect();
    let mut vec: Vec<_> = source.iter().collect();
    let iter = vec.con_drain(..).copied();
    let values: Vec<_> = iter.item_puller().collect();
    assert_eq!(values, source);
}

#[test]
fn cloned() {
    let source: Vec<_> = (2..6).collect();
    let mut vec: Vec<_> = source.iter().collect();
    let iter = vec.con_drain(..).cloned();
    let values: Vec<_> = iter.item_puller().collect();
    assert_eq!(values, source);
}
