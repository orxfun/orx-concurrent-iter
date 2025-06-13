pub struct ChunkPointers<T> {
    pub begin_idx: usize,
    pub first: *const T,
    pub last: *const T,
}

pub trait ArrayConIter {
    type Item;

    fn progress_and_get_chunk_pointers(
        &self,
        chunk_size: usize,
    ) -> Option<ChunkPointers<Self::Item>>;
}
