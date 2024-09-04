// #![allow(dead_code)]

// use orx_concurrent_bag::ConcurrentBag;
// use orx_concurrent_iter::iter::atomic_iter::*;
// use orx_concurrent_iter::*;
// use std::ops::Add;

// pub(crate) const ATOMIC_TEST_LEN: usize = 512;

// pub(crate) const ATOMIC_FETCH_N: [usize; 8] = [
//     1,
//     2,
//     4,
//     8,
//     ATOMIC_TEST_LEN / 2,
//     ATOMIC_TEST_LEN,
//     ATOMIC_TEST_LEN + 1,
//     ATOMIC_TEST_LEN * 2,
// ];

// pub(crate) fn atomic_fetch_one<T, A>(iter: A)
// where
//     A: AtomicIter<T>,
//     T: Add<usize, Output = usize> + Send + Sync,
// {
//     assert_eq!(0, iter.counter().current());

//     let mut i = 0;
//     while let Some(next) = iter.fetch_one() {
//         let value = next.value + 0usize;
//         assert_eq!(value, i);
//         i += 1;
//         assert_eq!(i, iter.counter().current());
//     }
// }

// pub(crate) fn atomic_fetch_n<T, A>(iter: A, n: usize)
// where
//     A: AtomicIter<T>,
//     T: Add<usize, Output = usize> + Send + Sync,
// {
//     assert_eq!(0, iter.counter().current());

//     let mut i = 0;

//     while let Some(chunk) = iter.fetch_n(n) {
//         for (j, value) in chunk.values.enumerate() {
//             let value = value + 0usize;
//             assert_eq!(value, chunk.begin_idx + j);
//             assert_eq!(value, i);

//             i += 1;
//         }
//     }
// }

// pub(crate) fn atomic_initial_len<T, A>(iter: A)
// where
//     A: AtomicIterWithInitialLen<T>,
//     T: Add<usize, Output = usize> + Send + Sync,
// {
//     assert_eq!(iter.initial_len(), ATOMIC_TEST_LEN);
// }

// pub(crate) fn test_values<C: ConcurrentIter>(num_threads: usize, len: usize, con_iter: C)
// where
//     C::Item: Add<usize, Output = usize>,
// {
//     let collected = ConcurrentBag::new();
//     let bag = &collected;
//     let iter = &con_iter;

//     std::thread::scope(|s| {
//         for _ in 0..num_threads {
//             s.spawn(move || {
//                 for value in iter.values() {
//                     bag.push(value + 0usize);
//                 }
//             });
//         }
//     });

//     assert_eq!(collected.len(), len);

//     let mut collected = collected.into_inner().to_vec();

//     collected.sort();
//     assert_eq!(collected, (0..len).collect::<Vec<_>>());
// }

// pub(crate) fn test_ids_and_values<C: ConcurrentIter>(num_threads: usize, len: usize, con_iter: C)
// where
//     C::Item: Add<usize, Output = usize>,
// {
//     let collected = ConcurrentBag::new();
//     let bag = &collected;
//     let iter = &con_iter;

//     std::thread::scope(|s| {
//         for _ in 0..num_threads {
//             s.spawn(move || {
//                 for (i, value) in iter.ids_and_values() {
//                     bag.push((i, value + 0usize));
//                 }
//             });
//         }
//     });

//     assert_eq!(collected.len(), len);

//     let mut collected = collected.into_inner().to_vec();
//     for (i, value) in &collected {
//         assert_eq!(i, value);
//     }

//     collected.sort();
//     assert_eq!(collected, (0..len).map(|x| (x, x)).collect::<Vec<_>>());
// }
