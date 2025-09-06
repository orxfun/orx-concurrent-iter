use crate::{
    ConcurrentIter, ExactSizeConcurrentIter,
    chain::{
        chunk_puller::ChainedChunkPuller, con_iter_known_len_i::ChainKnownLenI,
        con_iter_unknown_len_i::ChainUnknownLenI,
    },
};

pub enum Chain<I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    KnownLenI(ChainKnownLenI<I, J>),
    UnknownLenI(ChainUnknownLenI<I, J>),
}

impl<I, J> Chain<I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    pub(crate) fn new(i: I, j: J) -> Self {
        match i.try_get_len() {
            Some(len_i) => Self::KnownLenI(ChainKnownLenI::new(i, j, len_i)),
            None => Self::UnknownLenI(ChainUnknownLenI::new(i, j)),
        }
    }
}

impl<I, J> ConcurrentIter for Chain<I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    type Item = I::Item;

    type SequentialIter = core::iter::Chain<I::SequentialIter, J::SequentialIter>;

    type ChunkPuller<'i>
        = ChainedChunkPuller<'i, I, J>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        todo!()
    }

    fn skip_to_end(&self) {
        match self {
            Self::KnownLenI(k) => k.skip_to_end(),
            Self::UnknownLenI(u) => u.skip_to_end(),
        }
    }

    fn next(&self) -> Option<Self::Item> {
        match self {
            Self::KnownLenI(k) => k.next(),
            Self::UnknownLenI(u) => u.next(),
        }
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        match self {
            Self::KnownLenI(k) => k.next_with_idx(),
            Self::UnknownLenI(u) => u.next_with_idx(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::KnownLenI(k) => k.size_hint(),
            Self::UnknownLenI(u) => u.size_hint(),
        }
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        match self {
            Self::KnownLenI(k) => k.chunk_puller(chunk_size),
            Self::UnknownLenI(u) => u.chunk_puller(chunk_size),
        }
    }
}

impl<I, J> ExactSizeConcurrentIter for Chain<I, J>
where
    I: ExactSizeConcurrentIter,
    J: ExactSizeConcurrentIter<Item = I::Item>,
{
    fn len(&self) -> usize {
        match self {
            Self::KnownLenI(k) => k.len(),
            Self::UnknownLenI(u) => u.len(),
        }
    }
}
