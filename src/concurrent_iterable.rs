pub trait ConcurrentIterable {
    type Item: Send + Sync;

    type SeqIter: Iterator<Item = Self::Item>;
}
