use crate::{ChunkPuller, ConcurrentIter, IntoConcurrentIter};
use alloc::vec::Vec;

#[test]
fn enumerate_item() {
    let mut vec: Vec<_> = (2..6).collect();
    let slice = vec.as_mut_slice();
    let iter = slice.into_con_iter().enumerate();
    let mut j = 0;
    while let Some((i, x)) = iter.next() {
        assert_eq!((i, x), (j, &mut (j + 2)));
        j += 1;
    }

    let mut vec: Vec<_> = (2..6).collect();
    let slice = vec.as_mut_slice();
    let iter = slice.into_con_iter().enumerate();
    let mut j = 0;
    while let Some((i2, (i, x))) = iter.next_with_idx() {
        assert_eq!((i, x), (j, &mut (j + 2)));
        assert_eq!(i, i2);
        j += 1;
    }
}

#[test]
fn enumerate_item_puller() {
    let mut vec: Vec<_> = (2..6).collect();
    let slice = vec.as_mut_slice();
    let iter = slice.into_con_iter().enumerate();
    let puller = iter.item_puller();
    for (i, x) in puller {
        assert_eq!(x.clone(), 2 + i);
    }

    let mut vec: Vec<_> = (2..6).collect();
    let slice = vec.as_mut_slice();
    let iter = slice.into_con_iter().enumerate();
    let puller = iter.item_puller_with_idx();
    for (i, (j, x)) in puller {
        assert_eq!(i, j);
        assert_eq!(x.clone(), 2 + i);
    }
}

#[test]
fn enumerate_chunk_puller() {
    let mut vec: Vec<_> = (2..6).collect();
    let slice = vec.as_mut_slice();
    let iter = slice.into_con_iter().enumerate();

    let mut j = 0;

    let mut puller = iter.chunk_puller(2);
    while let Some(chunk) = puller.pull() {
        assert_eq!(chunk.len(), 2);
        for (i, x) in chunk {
            assert_eq!((i, x), (j, &mut (j + 2)));
            j += 1;
        }
    }
}
