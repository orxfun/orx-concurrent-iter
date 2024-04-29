use crate::{ConcurrentIter, ExactSizeConcurrentIter, NextChunk};

// ANY
pub(crate) fn any_for_each<I, F>(iter: &I, chunk_size: usize, f: F)
where
    I: ConcurrentIter,
    F: FnMut(I::Item),
{
    assert!(chunk_size > 0, "Chunk size must be positive.");
    let mut f = f;

    match chunk_size {
        1 => {
            while let Some(value) = iter.next() {
                f(value);
            }
        }
        _ => loop {
            let next = iter.next_chunk(chunk_size);
            let mut has_any = false;
            for value in next.values() {
                has_any = true;
                f(value);
            }

            if !has_any {
                break;
            }
        },
    }
}

pub(crate) fn any_for_each_with_ids<I, F>(iter: &I, chunk_size: usize, f: F)
where
    I: ConcurrentIter,
    F: FnMut(usize, I::Item),
{
    assert!(chunk_size > 0, "Chunk size must be positive.");
    let mut f = f;

    match chunk_size {
        1 => {
            while let Some(next) = iter.next_id_and_value() {
                f(next.idx, next.value);
            }
        }
        _ => loop {
            let next = iter.next_chunk(chunk_size);
            let begin_idx = next.begin_idx();
            let mut has_any = false;
            for (i, value) in next.values().enumerate() {
                has_any = true;
                f(begin_idx + i, value);
            }

            if !has_any {
                break;
            }
        },
    }
}

// EXACT
pub(crate) fn exact_for_each<I, F>(iter: &I, chunk_size: usize, f: F)
where
    I: ExactSizeConcurrentIter,
    F: FnMut(I::Item),
{
    assert!(chunk_size > 0, "Chunk size must be positive.");
    let mut f = f;

    match chunk_size {
        1 => {
            while let Some(value) = iter.next() {
                f(value);
            }
        }
        _ => {
            while let Some(next) = iter.next_exact_chunk(chunk_size) {
                next.values().for_each(&mut f);
            }
        }
    }
}

pub(crate) fn exact_for_each_with_ids<I, F>(iter: &I, chunk_size: usize, f: F)
where
    I: ExactSizeConcurrentIter,
    F: FnMut(usize, I::Item),
{
    assert!(chunk_size > 0, "Chunk size must be positive.");
    let mut f = f;

    match chunk_size {
        1 => {
            while let Some(next) = iter.next_id_and_value() {
                f(next.idx, next.value);
            }
        }
        _ => {
            while let Some(next) = iter.next_exact_chunk(chunk_size) {
                let begin_idx = next.begin_idx();
                for (i, value) in next.values().enumerate() {
                    f(begin_idx + i, value);
                }
            }
        }
    }
}
