use crate::ConcurrentIter;

pub(crate) fn for_each<I, F>(iter: &I, chunk_size: usize, fun: F)
where
    I: ConcurrentIter,
    F: FnMut(I::Item),
{
    assert!(chunk_size > 0, "Chunk size must be positive.");
    let mut f = fun;

    match chunk_size {
        1 => {
            while let Some(value) = iter.next() {
                f(value);
            }
        }
        _ => {
            let mut buffered_iter = iter.buffered_iter(chunk_size);
            while let Some(chunk) = buffered_iter.next() {
                chunk.values.for_each(&mut f);
            }
        }
    }
}

pub(crate) fn for_each_with_ids<I, F>(iter: &I, chunk_size: usize, fun: F)
where
    I: ConcurrentIter,
    F: FnMut(usize, I::Item),
{
    assert!(chunk_size > 0, "Chunk size must be positive.");
    let mut f = fun;

    match chunk_size {
        1 => {
            while let Some(next) = iter.next_id_and_value() {
                f(next.idx, next.value);
            }
        }
        _ => {
            let mut buffered_iter = iter.buffered_iter(chunk_size);
            while let Some(chunk) = buffered_iter.next() {
                let begin_idx = chunk.begin_idx;
                for (i, value) in chunk.values.enumerate() {
                    f(begin_idx + i, value);
                }
            }
        }
    }
}
