use orx_concurrent_iter::*;

fn test_threads_chunks<F>(test: F)
where
    F: Fn(usize, usize),
{
    let params = [(1, 1), (4, 1), (1, 4), (2, 64), (4, 32), (8, 16)];
    for (num_threads, batch) in params {
        test(num_threads, batch);
    }
}

fn predicate(value: &i32) -> bool {
    *value > 0 && value % 987 == 0
}

#[test]
fn early_exit_range() {
    fn test(num_threads: usize, batch: usize) {
        let range = 0i32..4096i32;
        let iter = range.into_con_iter();

        let found = par_find(iter, predicate, num_threads, batch);
        assert_eq!(found, Some((987, 987)));
    }

    test_threads_chunks(test)
}

#[test]
fn early_exit_vec() {
    fn test(num_threads: usize, batch: usize) {
        let vec: Vec<_> = (0..4096).collect();
        let iter = vec.into_con_iter();

        let found = par_find(iter, predicate, num_threads, batch);
        assert_eq!(found, Some((987, 987)));
    }

    test_threads_chunks(test)
}

#[test]
fn early_exit_slice_cloned() {
    fn test(num_threads: usize, batch: usize) {
        let vec: Vec<_> = (0..4096).collect();
        let iter = vec.as_slice().into_con_iter().cloned();

        let found = par_find(iter, predicate, num_threads, batch);
        assert_eq!(found, Some((987, 987)));
    }

    test_threads_chunks(test)
}

#[test]
fn early_exit_array() {
    fn test(num_threads: usize, batch: usize) {
        let mut array = [0i32; 4096];
        for (i, x) in array.iter_mut().enumerate() {
            *x = i as i32;
        }
        let iter = array.into_con_iter();

        let found = par_find(iter, predicate, num_threads, batch);
        assert_eq!(found, Some((987, 987)));
    }

    test_threads_chunks(test)
}

#[test]
fn early_exit_iter() {
    fn test(num_threads: usize, batch: usize) {
        let range = -1000i32..14096i32;
        let iter = range.skip(54).filter(|x| *x >= 0).take(4096);
        let iter = iter.into_con_iter();

        let found = par_find(iter, predicate, num_threads, batch);
        assert_eq!(found, Some((987, 987)));
    }

    test_threads_chunks(test)
}

#[test]
fn early_exit_slice() {
    fn test(num_threads: usize, batch: usize) {
        let vec: Vec<_> = (0..4096).collect();
        let iter = vec.as_slice().into_con_iter();

        let found = std::thread::scope(|s| {
            let mut handles = vec![];
            for _ in 0..num_threads {
                handles.push(s.spawn(|| {
                    if batch == 1 {
                        while let Some(next) = iter.next_id_and_value() {
                            if predicate(next.value) {
                                iter.skip_to_end();
                                return Some((next.idx, *next.value));
                            }
                        }
                    } else {
                        while let Some(chunk) = iter.next_chunk(batch) {
                            for (i, x) in chunk.values.enumerate() {
                                if predicate(x) {
                                    iter.skip_to_end();
                                    return Some((chunk.begin_idx + i, *x));
                                }
                            }
                        }
                    }
                    None
                }));
            }

            let collected: Vec<_> = handles
                .into_iter()
                .flat_map(|x| x.join().expect("-"))
                .collect();

            assert_eq!(collected.len(), 1, "early exit failed");
            collected[0]
        });

        assert_eq!(found, (987, 987));
    }

    test_threads_chunks(test)
}

fn par_find<I, P>(iter: I, predicate: P, num_threads: usize, batch: usize) -> Option<(usize, i32)>
where
    I: ConcurrentIter<Item = i32>,
    P: Fn(&i32) -> bool + Send + Sync,
{
    std::thread::scope(|s| {
        let mut handles = vec![];
        for _ in 0..num_threads {
            handles.push(s.spawn(|| {
                if batch == 1 {
                    while let Some(next) = iter.next_id_and_value() {
                        if predicate(&next.value) {
                            iter.skip_to_end();
                            return Some((next.idx, next.value));
                        }
                    }
                } else {
                    while let Some(chunk) = iter.next_chunk(batch) {
                        for (i, x) in chunk.values.enumerate() {
                            if predicate(&x) {
                                iter.skip_to_end();
                                return Some((chunk.begin_idx + i, x));
                            }
                        }
                    }
                }
                None
            }));
        }

        let results: Vec<_> = handles
            .into_iter()
            .flat_map(|x| x.join().expect("-"))
            .collect();

        assert_eq!(results.len(), 1, "early exit failed");

        results.into_iter().min_by_key(|x| x.0)
    })
}
