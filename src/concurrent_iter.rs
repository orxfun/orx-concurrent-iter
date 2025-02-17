pub trait ConcurrentIter {
    /// Type of the items that the iterator yields.
    type Item: Send + Sync;

    /// Inner type which is the source of the data to be iterated, which in addition is a regular sequential `Iterator`.
    type SeqIter: Iterator<Item = Self::Item>;

    type ChunksIter: ExactSizeIterator<Item = Self::Item>;

    fn into_seq_iter(self) -> Self::SeqIter;

    fn next(&self) -> Option<Self::Item>;

    fn skip_to_end(&self);

    fn in_chunks(&self, chunk_size: usize) -> Self::ChunksIter;

    // len

    /// Returns Some of the remaining length of the iterator if it is known; returns None otherwise.
    fn try_get_len(&self) -> Option<usize>;

    /// Returns Some of the initial length of the iterator when it was constructed if it is known; returns None otherwise.
    ///
    /// This method is often required and used for certain optimizations in parallel computing.
    fn try_get_initial_len(&self) -> Option<usize>;
}

// #[test]
// fn abc() {
//     fn xyz(iter: &impl ConcurrentIter<Item = u32>) {
//         let mut sum = 0u32;
//         let mut iter = iter.in_chunks(10);
//         while let Some(chunk) = iter.next() {
//             for x in chunk {
//                 sum += x;
//             }
//         }
//     }
// }
