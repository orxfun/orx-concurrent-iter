pub trait ConcurrentIterator {
    type Item: Send + Sync;

    type SeqIter: Iterator<Item = Self::Item>;

    fn into_seq_iter(self) -> Self::SeqIter;

    // iterate

    fn skip_to_end(&self);

    fn next(&self) -> Option<Self::Item>;

    fn next_with_idx(&self) -> Option<(usize, Self::Item)>;

    // len

    fn size_hint(&self) -> (usize, Option<usize>);

    fn try_get_len(&self) -> Option<usize> {
        match self.size_hint() {
            (_, None) => None,
            (_, Some(upper)) => Some(upper),
        }
    }
}
