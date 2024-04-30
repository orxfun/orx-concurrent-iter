use crate::{ConcurrentIter, ExactSizeConcurrentIter, NextChunk};

// ANY
pub(crate) fn any_fold<I, F, B>(iter: &I, chunk_size: usize, f: F, initial: B) -> B
where
    I: ConcurrentIter,
    F: FnMut(B, I::Item) -> B,
{
    assert!(chunk_size > 0, "Chunk size must be positive.");
    let mut f = f;

    let mut result = initial;

    match chunk_size {
        1 => {
            while let Some(value) = iter.next() {
                result = f(result, value);
            }
        }
        _ => loop {
            let next = iter.next_chunk(chunk_size);
            let mut has_any = false;
            for value in next.values() {
                has_any = true;
                result = f(result, value);
            }

            if !has_any {
                break;
            }
        },
    }

    result
}

// EXACT
pub(crate) fn exact_fold<I, F, B>(iter: &I, chunk_size: usize, f: F, initial: B) -> B
where
    I: ExactSizeConcurrentIter,
    F: FnMut(B, I::Item) -> B,
{
    assert!(chunk_size > 0, "Chunk size must be positive.");
    let mut f = f;

    let mut result = initial;

    match chunk_size {
        1 => {
            while let Some(value) = iter.next() {
                result = f(result, value);
            }
        }
        _ => {
            while let Some(next) = iter.next_exact_chunk(chunk_size) {
                for value in next.values() {
                    result = f(result, value);
                }
            }
        }
    }

    result
}
