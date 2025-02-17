use crate::{chunks_iter::ChunksIter, next::Next};

pub trait ConcurrentIter {
    /// Type of the items that the iterator yields.
    type Item;

    /// Inner type which is the source of the data to be iterated, which in addition is a regular sequential `Iterator`.
    type SeqIter: Iterator<Item = Self::Item>;

    type ChunksIter<'i>: ChunksIter<Item = Self::Item>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SeqIter;

    fn skip_to_end(&self);

    fn next<N: Next<Self::Item>>(&self) -> Option<N>;

    fn next_chunk<'i, N: Next<<Self::ChunksIter<'i> as ChunksIter>::Iter>>(
        &'i self,
        chunk_size: usize,
    ) -> Option<N> {
        self.in_chunks(chunk_size).pull()
    }

    fn in_chunks(&self, chunk_size: usize) -> Self::ChunksIter<'_>;
}

#[test]
fn abc() {
    fn xyz(iter: &impl ConcurrentIter<Item = String>) {
        let x: String = iter.next().unwrap();
        let y: (usize, String) = iter.next().unwrap();
    }
}
