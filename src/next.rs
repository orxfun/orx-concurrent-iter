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

impl<T: Clone> Next<&T> {
    /// Converts the next into one where the `value` is cloned.
    pub fn cloned(self) -> Next<T> {
        Next {
            idx: self.idx,
            value: self.value.clone(),
        }
    }
}

impl<T: Copy> Next<&T> {
    /// Converts the next into one where the `value` is copied.
    pub fn copied(self) -> Next<T> {
        Next {
            idx: self.idx,
            value: *self.value,
        }
    }
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

impl<'a, T: Clone, Iter> NextChunk<&'a T, Iter>
where
    Iter: ExactSizeIterator<Item = &'a T>,
{
    /// Converts the next into one where the `values` are cloned.
    pub fn cloned(self) -> NextChunk<T, core::iter::Cloned<Iter>> {
        NextChunk {
            begin_idx: self.begin_idx,
            values: self.values.cloned(),
        }
    }
}

impl<'a, T: Copy, Iter> NextChunk<&'a T, Iter>
where
    Iter: ExactSizeIterator<Item = &'a T>,
{
    /// Converts the next into one where the `values` are copied.
    pub fn copied(self) -> NextChunk<T, core::iter::Copied<Iter>> {
        NextChunk {
            begin_idx: self.begin_idx,
            values: self.values.copied(),
        }
    }
}
