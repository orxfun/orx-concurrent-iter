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
pub struct NextChunk<T, Iter>
where
    Iter: ExactSizeIterator<Item = T>,
{
    /// The index of the first element to be yielded by the `values` iterator.
    pub begin_idx: usize,

    /// Elements in the obtained chunk.
    pub values: Iter,
}
