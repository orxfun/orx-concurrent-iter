/// Result of a `next_id_and_value` call on a concurrent iterator which contains two bits of information:
/// * `idx`: index of the element in the iterator.
/// * `value`: of the element.
#[derive(Debug)]
pub struct Next<T> {
    /// Index of the element in the iterator.
    pub idx: usize,
    /// Value of the element.
    pub value: T,
}

/// A trait representing return types of a `next_chunk` call on a concurrent iterator.
pub trait NextChunk<T> {
    /// Type of the iterator yielding elements of the chunk.
    type ChunkIter: Iterator<Item = T>;

    /// Elements in the obtained chunk.
    fn values(self) -> Self::ChunkIter;

    /// The index of the first element to be yielded by the `values` iterator.
    fn begin_idx(&self) -> usize;
}

/// A `NextChunk` implementation with a begin index and any iterator, not necessarily with a known length.
#[derive(Debug)]
pub struct NextMany<T, IntoIter>
where
    IntoIter: IntoIterator<Item = T>,
    IntoIter::IntoIter: ExactSizeIterator<Item = T>,
{
    pub(crate) begin_idx: usize,
    pub(crate) values: IntoIter,
}

impl<T, Iter, IntoIter> NextChunk<T> for NextMany<T, IntoIter>
where
    Iter: Iterator<Item = T>,
    IntoIter: IntoIterator<Item = T, IntoIter = Iter>,
    IntoIter::IntoIter: ExactSizeIterator<Item = T>,
{
    type ChunkIter = Iter;

    /// Type of the iterator yielding elements of the chunk.
    #[inline(always)]
    fn values(self) -> Self::ChunkIter {
        self.values.into_iter()
    }

    /// The index of the first element to be yielded by the `values` iterator.
    #[inline(always)]
    fn begin_idx(&self) -> usize {
        self.begin_idx
    }
}

/// A `NextChunk` implementation with a begin index and an `ExactSizeIterator` iterator with known length.
#[derive(Debug)]
pub struct NextManyExact<T, Iter>
where
    Iter: ExactSizeIterator<Item = T>,
{
    pub(crate) begin_idx: usize,
    pub(crate) values: Iter,
}

impl<T, Iter: ExactSizeIterator<Item = T>> NextManyExact<T, Iter> {
    /// Exact length of the chunk which is less than or equal to the requested chunk size.
    pub fn exact_len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether or not the chunk is empty.
    pub fn is_empty(&self) -> bool {
        self.values.len() == 0
    }
}

impl<T, Iter: ExactSizeIterator<Item = T>> NextChunk<T> for NextManyExact<T, Iter> {
    type ChunkIter = Iter;

    #[inline(always)]
    fn values(self) -> Self::ChunkIter {
        self.values
    }

    #[inline(always)]
    fn begin_idx(&self) -> usize {
        self.begin_idx
    }
}
