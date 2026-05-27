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

    #[inline]
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

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::P(p) => p.fold(init, f),
            Self::Q(q) => q.fold(init, f),
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        match self {
            Self::P(p) => p.count(),
            Self::Q(q) => q.count(),
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
