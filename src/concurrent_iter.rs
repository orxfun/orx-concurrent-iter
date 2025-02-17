use crate::next::Next;

pub trait ConcurrentIter {
    /// Type of the items that the iterator yields.
    type Item;

    /// Inner type which is the source of the data to be iterated, which in addition is a regular sequential `Iterator`.
    type SeqIter: Iterator<Item = Self::Item>;

    // type ChunksIter: ExactSizeIterator<Item = Self::Item>;

    fn into_seq_iter(self) -> Self::SeqIter;

    fn next<N: Next<Self::Item>>(&self) -> Option<N>;

    fn skip_to_end(&self);

    // fn in_chunks(&self, chunk_size: usize) -> Self::ChunksIter;
}

#[test]
fn abc() {
    fn xyz(iter: &impl ConcurrentIter<Item = String>) {
        let x: String = iter.next().unwrap();
        let y: (usize, String) = iter.next().unwrap();
    }
}
