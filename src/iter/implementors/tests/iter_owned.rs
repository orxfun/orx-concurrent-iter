use crate::{
    ConIterOfIter, ConIterOfIterX, ConcurrentIter, ConcurrentIterX, IterIntoConcurrentIter,
};
use test_case::test_matrix;

const VEC_LEN: usize = 42;
const VEC_CAP: usize = 64;

#[derive(Clone, Copy, Debug)]
enum ConsumeRemaining {
    Leave,
    Next,
    SkipToEnd,
}

#[derive(Clone, Copy, Debug)]
enum Take {
    None,
    Some,
    All,
}
impl Take {
    fn take(self, vec_len: usize) -> usize {
        match self {
            Self::None => 0,
            Self::Some => vec_len / 3,
            Self::All => vec_len,
        }
    }
}

fn vec(len: usize, cap: usize) -> Vec<String> {
    let mut vec = Vec::with_capacity(cap);
    vec.extend((0..len).map(|i| i.to_string()));
    vec
}

#[test_matrix(
    [Take::None, Take::Some, Take::All]
)]
fn drop_without_next(take: Take) {
    let source = vec(VEC_LEN, VEC_CAP).into_iter();
    let len = source.len();
    let num_take = take.take(len);

    let con_iter: ConIterOfIter<String, _> = source.into_con_iter();
    for i in 0..num_take {
        let next = con_iter.next();
        assert_eq!(next, Some(i.to_string()));
    }
}

#[test_matrix(
    [Take::None, Take::Some, Take::All],
    [ConsumeRemaining::Leave, ConsumeRemaining::Next, ConsumeRemaining::SkipToEnd]
)]
fn drop_after_next(take: Take, remaining: ConsumeRemaining) {
    let source = vec(VEC_LEN, VEC_CAP).into_iter();
    let len = source.len();
    let num_take = take.take(len);

    let con_iter: ConIterOfIter<String, _> = source.into_con_iter();
    for i in 0..num_take {
        let next = con_iter.next();
        assert_eq!(next, Some(i.to_string()));
    }

    match remaining {
        ConsumeRemaining::Leave => {}
        ConsumeRemaining::Next => {
            for i in num_take..len {
                let next = con_iter.next();
                assert_eq!(next, Some(i.to_string()));
            }
        }
        ConsumeRemaining::SkipToEnd => con_iter.skip_to_end(),
    }
}

#[test_matrix(
    [Take::None, Take::Some, Take::All],
    [ConsumeRemaining::Leave, ConsumeRemaining::Next, ConsumeRemaining::SkipToEnd]
)]
fn drop_after_into_seq(take: Take, remaining: ConsumeRemaining) {
    let source = vec(VEC_LEN, VEC_CAP).into_iter();
    let len = source.len();
    let num_take = take.take(len);

    let con_iter: ConIterOfIter<String, _> = source.into_con_iter();
    for i in 0..num_take {
        let next = con_iter.next();
        assert_eq!(next, Some(i.to_string()));
    }

    let mut seq_iter = con_iter.into_seq_iter();

    match remaining {
        ConsumeRemaining::Leave => {}
        ConsumeRemaining::Next => {
            for i in num_take..len {
                let next = seq_iter.next();
                assert_eq!(next, Some(i.to_string()));
            }
            assert_eq!(seq_iter.next(), None);
        }
        ConsumeRemaining::SkipToEnd => {
            let _ = seq_iter.skip(len).count();
        }
    }
}

#[test_matrix(
    [Take::None, Take::Some, Take::All],
    [ConsumeRemaining::Leave, ConsumeRemaining::Next, ConsumeRemaining::SkipToEnd]
)]
fn drop_after_next_then_into_seq(take: Take, remaining: ConsumeRemaining) {
    let source = vec(VEC_LEN, VEC_CAP).into_iter();
    let len = source.len();
    let num_take = take.take(len);

    let con_iter: ConIterOfIter<String, _> = source.into_con_iter();
    for i in 0..num_take {
        let next = con_iter.next();
        assert_eq!(next, Some(i.to_string()));
    }

    let mut seq_iter = con_iter.into_seq_iter();

    match remaining {
        ConsumeRemaining::Leave => {}
        ConsumeRemaining::Next => {
            for i in num_take..len {
                let next = seq_iter.next();
                assert_eq!(next, Some(i.to_string()));
            }
            assert_eq!(seq_iter.next(), None);
        }
        ConsumeRemaining::SkipToEnd => {
            let _ = seq_iter.skip(len).count();
        }
    }
}

#[test_matrix(
    [true, false],
    [Take::None, Take::Some, Take::All],
    [ConsumeRemaining::Leave, ConsumeRemaining::Next, ConsumeRemaining::SkipToEnd]
)]
fn drop_after_next_chunk(consume_chunk: bool, take: Take, remaining: ConsumeRemaining) {
    let source = vec(VEC_LEN, VEC_CAP).into_iter();
    let len = source.len();
    let num_take = take.take(len);

    let con_iter: ConIterOfIter<String, _> = source.into_con_iter();

    let chunk = con_iter.next_chunk(num_take);
    if consume_chunk {
        if let Some(chunk) = chunk {
            for (i, next) in chunk.values.enumerate() {
                assert_eq!(next, i.to_string());
            }
        }
    }

    match remaining {
        ConsumeRemaining::Leave => {}
        ConsumeRemaining::Next => {
            for i in num_take..len {
                let next = con_iter.next();
                assert_eq!(next, Some(i.to_string()));
            }
            assert_eq!(con_iter.next(), None);
        }
        ConsumeRemaining::SkipToEnd => con_iter.skip_to_end(),
    }
}

#[test_matrix(
    [true, false],
    [Take::None, Take::Some, Take::All],
    [ConsumeRemaining::Leave, ConsumeRemaining::Next, ConsumeRemaining::SkipToEnd]
)]
fn drop_after_next_chunk_then_into_seq(
    consume_chunk: bool,
    take: Take,
    remaining: ConsumeRemaining,
) {
    let source = vec(VEC_LEN, VEC_CAP).into_iter();
    let len = source.len();
    let num_take = take.take(len);

    let con_iter: ConIterOfIter<String, _> = source.into_con_iter();

    {
        let chunk = con_iter.next_chunk(num_take);
        if consume_chunk {
            if let Some(chunk) = chunk {
                for (i, next) in chunk.values.enumerate() {
                    assert_eq!(next, i.to_string());
                }
            }
        }
    }

    let mut seq_iter = con_iter.into_seq_iter();
    match remaining {
        ConsumeRemaining::Leave => {}
        ConsumeRemaining::Next => {
            for i in num_take..len {
                let next = seq_iter.next();
                assert_eq!(next, Some(i.to_string()));
            }
            assert_eq!(seq_iter.next(), None);
        }
        ConsumeRemaining::SkipToEnd => {
            let _ = seq_iter.skip(len).count();
        }
    }
}

#[test_matrix(
    [true, false],
    [Take::None, Take::Some, Take::All],
    [ConsumeRemaining::Leave, ConsumeRemaining::Next, ConsumeRemaining::SkipToEnd]
)]
fn into_iter_x(consume_chunk: bool, take: Take, remaining: ConsumeRemaining) {
    let source = vec(VEC_LEN, VEC_CAP).into_iter();
    let len = source.len();
    let num_take = take.take(len);

    let con_iter: ConIterOfIter<String, _> = source.into_con_iter();
    let con_iter_x: ConIterOfIterX<String, _> = con_iter.into_concurrent_iter_x();

    {
        let chunk = con_iter_x.next_chunk_x(num_take);
        if consume_chunk {
            if let Some(chunk) = chunk {
                for (i, next) in chunk.enumerate() {
                    assert_eq!(next, i.to_string());
                }
            }
        }
    }

    let mut seq_iter = con_iter_x.into_seq_iter();
    match remaining {
        ConsumeRemaining::Leave => {}
        ConsumeRemaining::Next => {
            for i in num_take..len {
                let next = seq_iter.next();
                assert_eq!(next, Some(i.to_string()));
            }
            assert_eq!(seq_iter.next(), None);
        }
        ConsumeRemaining::SkipToEnd => {
            let _ = seq_iter.skip(len).count();
        }
    }
}
