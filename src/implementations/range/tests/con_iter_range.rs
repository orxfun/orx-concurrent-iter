use crate::{
    concurrent_iter::ConcurrentIter,
    enumeration::{Element, Enumerated, Enumeration, Regular},
    implementations::range::con_iter_range::ConIterRange,
    pullers::ChunkPuller,
    IntoConcurrentIter,
};
use core::{fmt::Debug, ops::Range};
use orx_concurrent_bag::ConcurrentBag;
use test_case::test_matrix;

#[cfg(miri)]
const N: usize = 125;
#[cfg(not(miri))]
const N: usize = 4735;

#[test]
fn enumeration() {
    let vec: Range<usize> = 0..6;

    let iter = vec.into_concurrent_iter();
    assert_eq!(iter.next(), Some(0));

    let enumerated = iter.enumerated();
    assert_eq!(enumerated.next(), Some((1, 1)));

    let iter = enumerated.not_enumerated();
    assert_eq!(iter.next(), Some(2));

    let enumerated = iter.enumerated();
    assert_eq!(enumerated.next(), Some((3, 3)));

    let iter = enumerated.not_enumerated();
    assert_eq!(iter.next(), Some(4));

    let enumerated = iter.enumerated();
    assert_eq!(enumerated.next(), Some((5, 5)));

    let iter = enumerated.not_enumerated();
    assert_eq!(iter.next(), None);

    let enumerated = iter.enumerated();
    assert_eq!(enumerated.next(), None);

    let iter = enumerated.not_enumerated();
    assert_eq!(iter.next(), None);

    let enumerated = iter.enumerated();
    assert_eq!(enumerated.next(), None);
}

#[test_matrix([Regular, Enumerated])]
fn size_hint<E: Enumeration>(_: E) {
    let mut n = 25;
    let iter = ConIterRange::<usize, E>::new(10..(10 + n));

    for _ in 0..10 {
        assert_eq!(iter.size_hint(), (n, Some(n)));
        let _ = iter.next();
        n -= 1;
    }

    let mut chunks_iter = iter.chunk_puller(7);

    assert_eq!(iter.size_hint(), (n, Some(n)));
    let _ = chunks_iter.pull();
    n -= 7;

    assert_eq!(iter.size_hint(), (n, Some(n)));
    let _ = chunks_iter.pull();
    assert_eq!(iter.size_hint(), (1, Some(1)));

    let _ = chunks_iter.pull();
    assert_eq!(iter.size_hint(), (0, Some(0)));

    let _ = chunks_iter.pull();
    assert_eq!(iter.size_hint(), (0, Some(0)));

    let _ = iter.next();
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test_matrix([Regular, Enumerated])]
fn size_hint_skip_to_end<E: Enumeration>(_: E) {
    let n = 25;
    let iter = ConIterRange::<usize, E>::new(10..(10 + n));

    for _ in 0..10 {
        let _ = iter.next();
    }
    let mut chunks_iter = iter.chunk_puller(7);
    let _ = chunks_iter.pull();

    assert_eq!(iter.size_hint(), (8, Some(8)));

    iter.skip_to_end();
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test_matrix([Regular, Enumerated], [1, 2, 4])]
fn empty_range<E: Enumeration>(_: E, nt: usize) {
    let iter = ConIterRange::<usize, E>::new(10..10);

    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                assert!(iter.next().is_none());
                assert!(iter.next().is_none());

                let mut puller = iter.chunk_puller(5);
                assert!(puller.pull().is_none());
                assert!(puller.pull().is_none());

                let mut iter = iter.chunk_puller(5).flattened();
                assert!(iter.next().is_none());
                assert!(iter.next().is_none());
            });
        }
    });
}

#[test_matrix([Regular, Enumerated], [0, 1, N], [1, 2, 4])]
fn next<E: Enumeration>(_: E, n: usize, nt: usize)
where
    for<'a> <E::Element as Element>::ElemOf<usize>: PartialEq + Ord + Debug,
{
    let iter = ConIterRange::<usize, E>::new(10..(10 + n));

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                while let Some(x) = iter.next() {
                    _ = iter.size_hint();
                    bag.push(x);
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..n).map(|i| E::new_element(i, i + 10)).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([Regular, Enumerated], [0, 1, N], [1, 2, 4])]
fn chunks_iter<K: Enumeration>(_: K, n: usize, nt: usize)
where
    for<'a> <K::Element as Element>::ElemOf<usize>: PartialEq + Ord + Debug,
{
    let iter = ConIterRange::<usize, K>::new(10..(10 + n));

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                let mut chunks_iter = iter.chunk_puller(7);

                while let Some((begin_idx, chunk)) = chunks_iter.pull().map(K::destruct_chunk) {
                    assert!(chunk.len() <= 7);
                    for x in chunk {
                        _ = iter.size_hint();
                        let value = K::new_element_from_begin_idx(begin_idx, x);
                        bag.push(value);
                    }
                }
            });
        }
    });

    let mut expected = vec![];
    for i in 0..n {
        let c = (i / 7) * 7;
        expected.push(K::new_element(c, i + 10));
    }
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([Regular, Enumerated], [0, 1, N], [1, 2, 4])]
fn chunks_iter_flattened<K: Enumeration>(_: K, n: usize, nt: usize)
where
    for<'a> <K::Element as Element>::ElemOf<usize>: PartialEq + Ord + Debug,
{
    let iter = ConIterRange::<usize, K>::new(10..(10 + n));

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                let chunks_iter = iter.chunk_puller(7).flattened();
                for x in chunks_iter {
                    _ = iter.size_hint();
                    bag.push(x);
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..n).map(|i| K::new_element(i, i + 10)).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([Regular, Enumerated], [0, 1, N], [1, 2, 4])]
fn skip_to_end<K: Enumeration>(_: K, n: usize, nt: usize) {
    let iter = ConIterRange::<usize, K>::new(10..(10 + n));
    let until = n / 2;

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    let con_num_spawned = &num_spawned;
    let con_bag = &bag;
    let con_iter = &iter;
    std::thread::scope(|s| {
        for t in 0..nt {
            s.spawn(move || {
                con_num_spawned.push(true);
                while con_num_spawned.len() < nt {} // allow all threads to be spawned

                match t % 2 {
                    0 => {
                        while let Some(num) = con_iter.next().map(K::Element::item_from_element) {
                            match num < until + 10 {
                                true => _ = con_bag.push(num),
                                false => con_iter.skip_to_end(),
                            }
                        }
                    }
                    _ => {
                        for num in con_iter
                            .chunk_puller(7)
                            .flattened()
                            .map(K::Element::item_from_element)
                        {
                            match num < until + 10 {
                                true => _ = con_bag.push(num),
                                false => con_iter.skip_to_end(),
                            }
                        }
                    }
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..until).map(|i| i + 10).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([Regular, Enumerated], [0, 1, N], [1, 2, 4], [0, N / 2, N])]
fn into_seq_iter<K: Enumeration>(_: K, n: usize, nt: usize, until: usize) {
    let iter = ConIterRange::<usize, K>::new(10..(10 + n));

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    let con_num_spawned = &num_spawned;
    let con_bag = &bag;
    let con_iter = &iter;
    if until > 0 {
        std::thread::scope(|s| {
            for t in 0..nt {
                s.spawn(move || {
                    con_num_spawned.push(true);
                    while con_num_spawned.len() < nt {} // allow all threads to be spawned

                    match t % 2 {
                        0 => {
                            while let Some(num) = con_iter.next().map(K::Element::item_from_element)
                            {
                                con_bag.push(num);
                                if num >= until + 10 {
                                    break;
                                }
                            }
                        }
                        _ => {
                            let mut iter = con_iter.chunk_puller(7);
                            while let Some((_, chunk)) = iter.pull().map(K::destruct_chunk) {
                                let mut do_break = false;
                                for num in chunk {
                                    con_bag.push(num);
                                    if num >= until + 10 {
                                        do_break = true;
                                    }
                                }
                                if do_break {
                                    break;
                                }
                            }
                        }
                    }
                });
            }
        });
    }

    let iter = iter.into_seq_iter();
    let remaining: Vec<usize> = iter.collect();
    let collected = bag.into_inner().to_vec();
    let mut all: Vec<_> = collected.into_iter().chain(remaining.into_iter()).collect();
    all.sort();

    let expected: Vec<_> = (0..n).map(|i| i + 10).collect();

    assert_eq!(all, expected);
}
