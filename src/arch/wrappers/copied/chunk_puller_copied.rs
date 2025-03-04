use crate::enumeration::{Element, Enumeration};
use crate::pullers::{ChunkPuller, PulledChunkIter};
use core::iter::Copied;
use core::marker::PhantomData;

pub struct ChunkPullerCopied<'a, T, E, P>
where
    T: Send + Sync + Copy + 'a,
    E: Enumeration,
    P: ChunkPuller<E, ChunkItem = &'a T>,
{
    chunks_puller: P,
    phantom: PhantomData<&'a (T, E)>,
}

impl<'a, T, E, P> ChunkPullerCopied<'a, T, E, P>
where
    T: Send + Sync + Copy + 'a,
    E: Enumeration,
    P: ChunkPuller<E, ChunkItem = &'a T>,
{
    pub(super) fn new(chunks_puller: P) -> Self {
        Self {
            chunks_puller,
            phantom: PhantomData,
        }
    }
}

impl<'a, T, E, P> ChunkPuller<E> for ChunkPullerCopied<'a, T, E, P>
where
    T: Send + Sync + Copy + 'a,
    E: Enumeration,
    P: ChunkPuller<E, ChunkItem = &'a T>,
{
    type ChunkItem = T;

    type Iter<'c>
        = Copied<P::Iter<'c>>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.chunks_puller.chunk_size()
    }

    fn pull(&mut self) -> Option<<<E as Enumeration>::Element as Element>::IterOf<Self::Iter<'_>>> {
        self.chunks_puller
            .pull()
            .map(<<E as Enumeration>::Element as Element>::copied_iter)
    }

    fn pulli(&mut self) -> Option<PulledChunkIter<Self::Iter<'_>, E>> {
        let a: PulledChunkIter<<P as ChunkPuller<E>>::Iter<'_>, E> = self.chunks_puller.pulli()?;
        // let b = a.copied();
        // self.chunks_puller.pulli().map(|x| x.copied())
        None
    }
}
