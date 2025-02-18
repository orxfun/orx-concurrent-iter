use crate::{
    chunk_puller::ChunkPuller,
    concurrent_iter::ConcurrentIter,
    enumeration::{Element, Enumerated, Enumeration, Regular},
    implementations::slice::con_iter_slice_ref::ConIterSliceRef,
};
use core::fmt::Debug;
use orx_concurrent_bag::ConcurrentBag;
use test_case::test_matrix;

#[cfg(miri)]
const N: usize = 125;
#[cfg(not(miri))]
const N: usize = 5442;

#[test_matrix([Regular, Enumerated], [1, 2, 4])]
fn empty_slice<K: Enumeration>(_: K, nt: usize) {
    let vec = Vec::<String>::new();
    let slice = vec.as_slice();
    let con_iter = ConIterSliceRef::<String, K>::new(slice);

    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                assert!(con_iter.next().is_none());
                assert!(con_iter.next().is_none());

                assert!(con_iter.next_chunk(4).is_none());
                assert!(con_iter.next_chunk(4).is_none());

                let mut puller = con_iter.chunks_iter(5);
                assert!(puller.next().is_none());

                let mut iter = con_iter.chunks_iter(5).flattened();
                assert!(iter.next().is_none());
            });
        }
    });
}

#[test_matrix([Regular, Enumerated], [1, 2, 4])]
fn next<K: Enumeration>(_: K, nt: usize)
where
    for<'a> <K::Element as Element>::ElemOf<&'a String>: PartialEq + Ord + Debug,
{
    let vec: Vec<_> = (0..N).map(|x| (x + 10).to_string()).collect();
    let slice = vec.as_slice();
    let iter = ConIterSliceRef::<String, K>::new(slice);

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                let mut i = 0;
                while let Some(x) = iter.next() {
                    i += 1;
                    bag.push(x);
                }

                assert!(i > 0);
            });
        }
    });

    let mut expected: Vec<_> = (0..N).map(|i| K::new_element(i, &slice[i])).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([Regular, Enumerated], [1, 2, 4])]
fn chunks_iter<K: Enumeration>(_: K, nt: usize)
where
    for<'a> <K::Element as Element>::ElemOf<&'a String>: PartialEq + Ord + Debug,
{
    let vec: Vec<_> = (0..N).map(|x| (x + 10).to_string()).collect();
    let slice = vec.as_slice();
    let iter = ConIterSliceRef::<String, K>::new(slice);

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                let chunks_iter = iter.chunks_iter(7);

                let mut i = 0;
                for (begin_idx, chunk) in chunks_iter.map(K::destruct_chunk) {
                    assert!(chunk.len() <= 7);
                    for x in chunk {
                        i += 1;
                        let value = K::new_element_from_begin_idx(begin_idx, x);
                        bag.push(value);
                    }
                }

                assert!(i > 0);
            });
        }
    });

    let mut expected = vec![];
    for i in 0..N {
        let c = (i / 7) * 7;
        expected.push(K::new_element(c, &slice[i]));
    }
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([Regular, Enumerated], [1, 2, 4])]
fn chunks_iter_flattened<K: Enumeration>(_: K, nt: usize)
where
    for<'a> <K::Element as Element>::ElemOf<&'a String>: PartialEq + Ord + Debug,
{
    let vec: Vec<_> = (0..N).map(|x| (x + 10).to_string()).collect();
    let slice = vec.as_slice();
    let iter = ConIterSliceRef::<String, K>::new(slice);

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                let chunks_iter = iter.chunks_iter(7).flattened();

                let mut i = 0;
                for x in chunks_iter {
                    i += 1;
                    bag.push(x);
                }

                assert!(i > 0);
            });
        }
    });

    let mut expected: Vec<_> = (0..N).map(|i| K::new_element(i, &slice[i])).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}
