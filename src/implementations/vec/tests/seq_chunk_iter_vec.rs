use crate::implementations::vec::seq_chunk_iter_vec::SeqChunksIterVec;
use core::mem::ManuallyDrop;

#[test]
fn empty() {
    let mut iter = SeqChunksIterVec::<String>::default();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
}

#[test]
fn all_consumed() {
    let n = 33;
    let cap = n + 19;
    let mut vec = Vec::with_capacity(cap);
    for i in 0..n {
        vec.push(i.to_string());
    }

    let (vec_len, vec_cap) = (vec.len(), vec.capacity());
    let first = vec.as_ptr();
    let last = unsafe { first.add(vec_len - 1) };
    let _ = ManuallyDrop::new(vec);

    let iter = SeqChunksIterVec::new(false, first, last);
    for (i, x) in iter.enumerate() {
        assert_eq!(x, i.to_string());
    }

    let _vec_to_drop = unsafe { Vec::from_raw_parts(first as *mut String, 0, vec_cap) };
}

#[test]
fn partially_consumed() {
    let n = 33;
    let cap = n + 19;
    let mut vec = Vec::with_capacity(cap);
    for i in 0..n {
        vec.push(i.to_string());
    }

    let (vec_len, vec_cap) = (vec.len(), vec.capacity());
    let first = vec.as_ptr();
    let last = unsafe { first.add(vec_len - 1) };
    let _ = ManuallyDrop::new(vec);

    {
        let mut iter = SeqChunksIterVec::new(false, first, last);
        for i in 0..(n / 2) {
            let x = iter.next().unwrap();
            assert_eq!(x, i.to_string());
        }
    }

    let _vec_to_drop = unsafe { Vec::from_raw_parts(first as *mut String, 0, vec_cap) };
}
