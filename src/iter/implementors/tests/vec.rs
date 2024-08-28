use test_case::test_matrix;

use crate::{ConIterOfVec, ConcurrentIter, IntoConcurrentIter};

fn vec(len: usize, cap: usize) -> Vec<String> {
    let mut vec = Vec::with_capacity(cap);
    vec.extend((0..len).map(|i| i.to_string()));
    vec
}

#[test]
fn drop_without_next() {
    let source = vec(42, 64);
    let _con_iter: ConIterOfVec<String> = source.into_con_iter();
}

#[test_matrix([true, false])]
fn drop_after_next(consume_remaining: bool) {
    let num_take = 11;

    let source = vec(42, 64);
    let len = source.len();

    let con_iter: ConIterOfVec<String> = source.into_con_iter();
    for i in 0..num_take {
        let next = con_iter.next();
        assert_eq!(next, Some(i.to_string()));
    }

    if consume_remaining {
        for i in num_take..len {
            let next = con_iter.next();
            assert_eq!(next, Some(i.to_string()));
        }
    }
}

#[test_matrix([true, false])]
fn drop_after_into_seq(consume_remaining: bool) {
    let source = vec(42, 64);
    let len = source.len();

    let source: Vec<_> = (0..len).map(|x| x.to_string()).collect();
    let con_iter: ConIterOfVec<String> = source.into_con_iter();
    let mut seq_iter = con_iter.into_seq_iter();

    if consume_remaining {
        for i in 0..len {
            let next = seq_iter.next();
            assert_eq!(next, Some(i.to_string()));
        }
        assert_eq!(seq_iter.next(), None);
    }
}

#[test_matrix([true, false])]
fn drop_after_next_then_into_seq(consume_remaining: bool) {
    let num_take = 11;

    let source = vec(42, 64);
    let len = source.len();

    let con_iter: ConIterOfVec<String> = source.into_con_iter();
    for i in 0..num_take {
        let next = con_iter.next();
        assert_eq!(next, Some(i.to_string()));
    }

    let mut seq_iter = con_iter.into_seq_iter();

    if consume_remaining {
        for i in num_take..len {
            let next = seq_iter.next();
            assert_eq!(next, Some(i.to_string()));
        }
        assert_eq!(seq_iter.next(), None);
    }
}

#[test_matrix(
    [true, false],
    [true, false]
)]
fn drop_after_next_chunk(consume_chunk: bool, consume_remaining: bool) {
    let num_take = 11;

    let source = vec(42, 64);
    let len = source.len();

    let con_iter: ConIterOfVec<String> = source.into_con_iter();

    let chunk = con_iter.next_chunk(num_take);
    if consume_chunk {
        if let Some(chunk) = chunk {
            for (i, next) in chunk.values.enumerate() {
                assert_eq!(next, i.to_string());
            }
        }
    }

    if consume_remaining {
        for i in num_take..len {
            let next = con_iter.next();
            assert_eq!(next, Some(i.to_string()));
        }

        assert_eq!(con_iter.next(), None);
    }
}

#[test_matrix(
    [true, false],
    [true, false]
)]
fn drop_after_next_chunk_then_into_seq(consume_chunk: bool, consume_remaining: bool) {
    let num_take = 11;

    let source = vec(42, 64);
    let len = source.len();

    let con_iter: ConIterOfVec<String> = source.into_con_iter();

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
    if consume_remaining {
        for i in num_take..len {
            let next = seq_iter.next();
            assert_eq!(next, Some(i.to_string()));
        }

        assert_eq!(seq_iter.next(), None);
    }
}
