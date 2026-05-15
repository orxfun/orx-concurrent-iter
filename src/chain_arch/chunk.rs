pub enum ChunkOfEither<P, Q>
where
    P: ExactSizeIterator,
    Q: ExactSizeIterator<Item = P::Item>,
{
    P(P),
    Q(Q),
}

impl<P, Q> Default for ChunkOfEither<P, Q>
where
    P: ExactSizeIterator + Default,
    Q: ExactSizeIterator<Item = P::Item>,
{
    fn default() -> Self {
        Self::P(Default::default())
    }
}

impl<P, Q> Iterator for ChunkOfEither<P, Q>
where
    P: ExactSizeIterator,
    Q: ExactSizeIterator<Item = P::Item>,
{
    type Item = P::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::P(p) => p.next(),
            Self::Q(q) => q.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::P(p) => p.size_hint(),
            Self::Q(q) => q.size_hint(),
        }
    }
}

impl<P, Q> ExactSizeIterator for ChunkOfEither<P, Q>
where
    P: ExactSizeIterator,
    Q: ExactSizeIterator<Item = P::Item>,
{
    fn len(&self) -> usize {
        match self {
            Self::P(p) => p.len(),
            Self::Q(q) => q.len(),
        }
    }
}
