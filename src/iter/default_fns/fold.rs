use crate::{iter::con_iter_x::ConcurrentIterX, ConcurrentIter};

pub(crate) fn fold<I, Fold, B>(iter: &I, chunk_size: usize, fold: Fold, neutral: B) -> B
where
    I: ConcurrentIter,
    Fold: FnMut(B, I::Item) -> B,
{
    assert!(chunk_size > 0, "Chunk size must be positive.");
    let mut f = fold;

    let mut result = neutral;

    match chunk_size {
        1 => {
            while let Some(value) = iter.next() {
                result = f(result, value);
            }
        }
        _ => {
            let mut buffered_iter = iter.buffered_iter(chunk_size);
            while let Some(chunk) = buffered_iter.next() {
                for value in chunk.values {
                    result = f(result, value);
                }
            }
        }
    }

    result
}

pub(crate) fn fold_x<I, Fold, B>(iter: &I, chunk_size: usize, fold: Fold, neutral: B) -> B
where
    I: ConcurrentIterX,
    Fold: FnMut(B, I::Item) -> B,
{
    assert!(chunk_size > 0, "Chunk size must be positive.");
    let mut f = fold;

    let mut result = neutral;

    match chunk_size {
        1 => {
            while let Some(value) = iter.next() {
                result = f(result, value);
            }
        }
        _ => {
            let mut buffered_iter = iter.buffered_iter(chunk_size);
            while let Some(chunk) = buffered_iter.next() {
                for value in chunk {
                    result = f(result, value);
                }
            }
        }
    }

    result
}
