use orx_concurrent_bag::*;
use orx_concurrent_iter::*;
use orx_iterable::Collection;
use std::fmt::Debug;
use test_case::test_matrix;

fn concurrent_iter<I>(num_threads: usize, batch: usize, con_iter: I) -> Vec<I::Item>
where
    I: ConcurrentIter + Send + Sync,
    I::Item: Send + Sync + Clone + Copy + Debug + PartialEq,
{
    let collected = ConcurrentBag::new();
    let bag = &collected;

    let iter = &con_iter;

    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(move || match batch == 1 {
                true => {
                    while let Some((idx, value)) = iter.next_with_idx() {
                        bag.push((idx, value));
                    }
                }
                false => {
                    let mut puller = iter.chunk_puller(batch);
                    while let Some((begin_idx, chunk)) = puller.pull_with_idx() {
                        for (i, value) in chunk.enumerate() {
                            bag.push((begin_idx + i, value));
                        }
                    }
                }
            });
        }
    });

    let mut vec: Vec<_> = collected.into_inner().iter().copied().collect();
    vec.sort_by_key(|x| x.0);
    vec.iter().map(|x| x.1).collect()
}

fn concurrent_iter_exact_len<I: ConcurrentIter>(iter: I, expected_len: usize, batch: usize) {
    assert_eq!(iter.try_get_len(), Some(expected_len));
    let mut remaining = expected_len;

    match batch == 1 {
        true => {
            while let Some(_) = iter.next() {
                remaining -= 1;
                assert_eq!(iter.try_get_len(), Some(remaining));
            }
            assert_eq!(iter.try_get_len(), Some(0));
            _ = iter.next();
            assert_eq!(iter.try_get_len(), Some(0));
        }
        false => {
            let mut puller = iter.chunk_puller(batch);
            while let Some(chunk) = puller.pull() {
                remaining -= chunk.len();
                assert_eq!(iter.try_get_len(), Some(remaining));
            }
            assert_eq!(iter.try_get_len(), Some(0));
            _ = puller.pull();
            assert_eq!(iter.try_get_len(), Some(0));
        }
    }
}

#[test_matrix(
    [1, 4, 1024, 4*1024],
    [1, 2, 8],
    [1, 4, 64, 1024]
)]
fn con_iter_slice(len: usize, num_threads: usize, batch: usize) {
    let source: Vec<_> = (0..len).collect();

    let clone = source.clone();
    let slice = clone.as_slice();

    let collected: Vec<&usize> = concurrent_iter(num_threads, batch, slice.con_iter());
    assert_eq!(source, collected.into_iter().copied().collect::<Vec<_>>());

    concurrent_iter_exact_len(clone.clone().as_slice().con_iter(), len, 1);
    concurrent_iter_exact_len(clone.clone().as_slice().con_iter(), len, 32);
    concurrent_iter_exact_len(clone.clone().as_slice().con_iter(), len, 33);
}

#[test_matrix(
    [1, 4, 1024, 4*1024],
    [1, 2, 8],
    [1, 4, 64, 1024]
)]
fn con_iter_vec(len: usize, num_threads: usize, batch: usize) {
    let source: Vec<_> = (0..len).collect();

    let collected: Vec<usize> = concurrent_iter(num_threads, batch, source.clone().into_con_iter());
    assert_eq!(source, collected.into_iter().collect::<Vec<_>>());

    concurrent_iter_exact_len(source.clone().into_con_iter(), len, 1);
    concurrent_iter_exact_len(source.clone().into_con_iter(), len, 32);
    concurrent_iter_exact_len(source.clone().into_con_iter(), len, 33);
}

#[test_matrix(
    [1, 4, 1024, 4*1024],
    [1, 2, 8],
    [1, 4, 64, 1024]
)]
fn con_iter_iter(len: usize, num_threads: usize, batch: usize) {
    let source: Vec<_> = (0..len).collect();

    let clone = source.clone();

    let collected: Vec<&usize> =
        concurrent_iter(num_threads, batch, clone.iter().iter_into_con_iter());
    assert_eq!(source, collected.into_iter().copied().collect::<Vec<_>>());

    concurrent_iter_exact_len(clone.iter().iter_into_con_iter(), len, 1);
    concurrent_iter_exact_len(clone.iter().iter_into_con_iter(), len, 32);
    concurrent_iter_exact_len(clone.iter().iter_into_con_iter(), len, 33);
    concurrent_iter_exact_len(clone.iter().map(|x| x * 2).iter_into_con_iter(), len, 1);
    concurrent_iter_exact_len(clone.iter().map(|x| x * 2).iter_into_con_iter(), len, 32);
    concurrent_iter_exact_len(clone.iter().map(|x| x * 2).iter_into_con_iter(), len, 33);
}
