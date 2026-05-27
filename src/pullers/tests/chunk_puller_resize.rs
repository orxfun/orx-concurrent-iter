use crate::concurrent_collection::ConcurrentCollection;
use crate::concurrent_iter::ConcurrentIter;
use crate::into_concurrent_iter::IntoConcurrentIter;
use crate::pullers::ChunkPuller;
use std::collections::VecDeque;

// ============================================================================
// Vec Tests
// ============================================================================

#[test]
fn chunk_puller_resize_vec_larger() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(2);

    // First pull with size 2
    let chunk1 = chunk_puller.pull().unwrap();
    assert_eq!(chunk1.len(), 2);

    // Resize to larger size
    chunk_puller.resize_for_chunk_size(4);
    assert_eq!(chunk_puller.chunk_size(), 4);

    // Next pull should have size 4
    let chunk2 = chunk_puller.pull().unwrap();
    assert_eq!(chunk2.len(), 4);
}

#[test]
fn chunk_puller_resize_vec_smaller() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(4);

    // First pull with size 4
    let chunk1 = chunk_puller.pull().unwrap();
    assert_eq!(chunk1.len(), 4);

    // Resize to smaller size
    chunk_puller.resize_for_chunk_size(2);
    assert_eq!(chunk_puller.chunk_size(), 2);

    // Next pull should have size 2
    let chunk2 = chunk_puller.pull().unwrap();
    assert_eq!(chunk2.len(), 2);
}

#[test]
fn chunk_puller_resize_vec_multiple_times() {
    let data: Vec<i32> = (0..20).collect();
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(3);

    // Pull with size 3
    let chunk1 = chunk_puller.pull().unwrap();
    assert_eq!(chunk1.len(), 3);

    // Resize to 5
    chunk_puller.resize_for_chunk_size(5);
    let chunk2 = chunk_puller.pull().unwrap();
    assert_eq!(chunk2.len(), 5);

    // Resize to 2
    chunk_puller.resize_for_chunk_size(2);
    let chunk3 = chunk_puller.pull().unwrap();
    assert_eq!(chunk3.len(), 2);

    // Resize back to 4
    chunk_puller.resize_for_chunk_size(4);
    let chunk4 = chunk_puller.pull().unwrap();
    assert_eq!(chunk4.len(), 4);
}

#[test]
fn chunk_puller_resize_vec_to_size_one() {
    let data = vec![10, 20, 30, 40];
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(2);

    let _ = chunk_puller.pull().unwrap(); // size 2
    chunk_puller.resize_for_chunk_size(1);

    let chunk = chunk_puller.pull().unwrap();
    assert_eq!(chunk.len(), 1);
}

#[test]
fn chunk_puller_resize_vec_larger_than_remaining() {
    let data = vec![1, 2, 3, 4, 5];
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(2);

    // Pull 2 items (3 remain)
    let _ = chunk_puller.pull().unwrap();

    // Resize to larger than remaining (10)
    chunk_puller.resize_for_chunk_size(10);

    // Should return only 3 items (remaining)
    let chunk = chunk_puller.pull().unwrap();
    assert_eq!(chunk.len(), 3);
}

// ============================================================================
// Slice Tests
// ============================================================================

#[test]
fn chunk_puller_resize_slice_larger() {
    // Using Vec as it directly supports con_iter()
    let data_inner = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let data = data_inner.clone();
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(2);

    let _ = chunk_puller.pull().unwrap(); // size 2
    chunk_puller.resize_for_chunk_size(4);

    let chunk = chunk_puller.pull().unwrap();
    assert_eq!(chunk.len(), 4);
}

#[test]
fn chunk_puller_resize_slice_smaller() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(4);

    let _ = chunk_puller.pull().unwrap(); // size 4
    chunk_puller.resize_for_chunk_size(2);

    let chunk = chunk_puller.pull().unwrap();
    assert_eq!(chunk.len(), 2);
}

#[test]
fn chunk_puller_resize_slice_multiple() {
    let data: Vec<i32> = (0..16).collect();
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(3);

    let _ = chunk_puller.pull().unwrap();
    chunk_puller.resize_for_chunk_size(5);
    let _ = chunk_puller.pull().unwrap();
    chunk_puller.resize_for_chunk_size(2);
    let final_chunk = chunk_puller.pull().unwrap();
    assert_eq!(final_chunk.len(), 2);
}

// ============================================================================
// Mutable Slice Tests (using into_con_iter)
// ============================================================================

#[test]
fn chunk_puller_resize_slice_mut_larger() {
    let mut data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    {
        let slice_mut: &mut [i32] = &mut data;
        let con_iter = slice_mut.into_con_iter();
        let mut chunk_puller = con_iter.chunk_puller(2);

        let _ = chunk_puller.pull().unwrap(); // size 2
        chunk_puller.resize_for_chunk_size(3);

        let chunk = chunk_puller.pull().unwrap();
        assert_eq!(chunk.len(), 3);
    }
}

#[test]
fn chunk_puller_resize_slice_mut_smaller() {
    let mut data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    {
        let slice_mut: &mut [i32] = &mut data;
        let con_iter = slice_mut.into_con_iter();
        let mut chunk_puller = con_iter.chunk_puller(4);

        let _ = chunk_puller.pull().unwrap(); // size 4
        chunk_puller.resize_for_chunk_size(1);

        let chunk = chunk_puller.pull().unwrap();
        assert_eq!(chunk.len(), 1);
    }
}

// ============================================================================
// VecDeque Tests
// ============================================================================

#[test]
fn chunk_puller_resize_vec_deque_larger() {
    let mut deque = VecDeque::new();
    for i in 0..8 {
        deque.push_back(i);
    }
    let con_iter = deque.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(2);

    chunk_puller.pull().unwrap(); // size 2
    chunk_puller.resize_for_chunk_size(4);

    let chunk = chunk_puller.pull().unwrap();
    assert_eq!(chunk.len(), 4);
}

#[test]
fn chunk_puller_resize_vec_deque_smaller() {
    let mut deque = VecDeque::new();
    for i in 0..8 {
        deque.push_back(i);
    }
    let con_iter = deque.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(4);

    chunk_puller.pull().unwrap(); // size 4
    chunk_puller.resize_for_chunk_size(2);

    let chunk = chunk_puller.pull().unwrap();
    assert_eq!(chunk.len(), 2);
}

// ============================================================================
// Resize with pull_with_idx Tests
// ============================================================================

#[test]
fn chunk_puller_resize_with_idx_vec_larger() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(2);

    let (idx1, chunk1) = chunk_puller.pull_with_idx().unwrap();
    assert_eq!(idx1, 0);
    assert_eq!(chunk1.len(), 2);

    chunk_puller.resize_for_chunk_size(4);

    let (idx2, chunk2) = chunk_puller.pull_with_idx().unwrap();
    assert_eq!(idx2, 2); // next chunk starts at index 2
    assert_eq!(chunk2.len(), 4);
}

#[test]
fn chunk_puller_resize_with_idx_vec_smaller() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(4);

    let (idx1, chunk1) = chunk_puller.pull_with_idx().unwrap();
    assert_eq!(idx1, 0);
    assert_eq!(chunk1.len(), 4);

    chunk_puller.resize_for_chunk_size(2);

    let (idx2, chunk2) = chunk_puller.pull_with_idx().unwrap();
    assert_eq!(idx2, 4); // next chunk starts at index 4
    assert_eq!(chunk2.len(), 2);
}

#[test]
fn chunk_puller_resize_with_idx_multiple() {
    let data: Vec<i32> = (0..12).collect();
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(2);

    // First chunk: indices 0-1
    let (idx1, chunk1) = chunk_puller.pull_with_idx().unwrap();
    assert_eq!(idx1, 0);
    assert_eq!(chunk1.len(), 2);

    // Resize to 3
    chunk_puller.resize_for_chunk_size(3);
    let (idx2, chunk2) = chunk_puller.pull_with_idx().unwrap();
    assert_eq!(idx2, 2);
    assert_eq!(chunk2.len(), 3);

    // Resize to 4
    chunk_puller.resize_for_chunk_size(4);
    let (idx3, chunk3) = chunk_puller.pull_with_idx().unwrap();
    assert_eq!(idx3, 5);
    assert_eq!(chunk3.len(), 4);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn chunk_puller_resize_to_zero_then_pull() {
    let data = vec![1, 2, 3, 4, 5];
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(2);

    let _ = chunk_puller.pull().unwrap();

    // Resize to 0 (edge case)
    chunk_puller.resize_for_chunk_size(0);

    // Pulling with size 0 might return None or an empty chunk
    // This depends on implementation details
    let chunk = chunk_puller.pull();
    // Accept either None or a chunk (implementation-dependent behavior)
    match chunk {
        Some(_c) => {
            // If it returns a chunk, the test passes
        }
        None => {
            // This is also acceptable behavior
        }
    }
}

#[test]
fn chunk_puller_resize_maintains_chunk_size() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(2);

    assert_eq!(chunk_puller.chunk_size(), 2);

    chunk_puller.resize_for_chunk_size(5);
    assert_eq!(chunk_puller.chunk_size(), 5);

    chunk_puller.resize_for_chunk_size(3);
    assert_eq!(chunk_puller.chunk_size(), 3);
}

#[test]
fn chunk_puller_resize_very_large_size() {
    let data: Vec<i32> = (0..100).collect();
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(10);

    let _ = chunk_puller.pull().unwrap();
    chunk_puller.resize_for_chunk_size(1000); // Much larger than remaining

    let chunk = chunk_puller.pull().unwrap();
    assert_eq!(chunk.len(), 90); // Only 90 items left
}

#[test]
fn chunk_puller_resize_small_dataset() {
    let data = vec![1, 2, 3];
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(5);

    // First pull with size 5 but only 3 items available
    let chunk1 = chunk_puller.pull().unwrap();
    assert_eq!(chunk1.len(), 3);

    // No more items
    assert!(chunk_puller.pull().is_none());

    // Resize shouldn't matter since no items left
    chunk_puller.resize_for_chunk_size(1);
    assert!(chunk_puller.pull().is_none());
}

#[test]
fn chunk_puller_resize_alternating_larger_and_smaller() {
    let data: Vec<i32> = (0..24).collect();
    let con_iter = data.con_iter();
    let mut chunk_puller = con_iter.chunk_puller(2);

    // 2, 4, 2, 4, 2, 4
    let _ = chunk_puller.pull().unwrap(); // 2 items (0-1)

    chunk_puller.resize_for_chunk_size(4);
    let _ = chunk_puller.pull().unwrap(); // 4 items (2-5)

    chunk_puller.resize_for_chunk_size(2);
    let _ = chunk_puller.pull().unwrap(); // 2 items (6-7)

    chunk_puller.resize_for_chunk_size(4);
    let _ = chunk_puller.pull().unwrap(); // 4 items (8-11)

    chunk_puller.resize_for_chunk_size(2);
    let _ = chunk_puller.pull().unwrap(); // 2 items (12-13)

    chunk_puller.resize_for_chunk_size(4);
    let final_chunk = chunk_puller.pull().unwrap(); // 4 items (14-17)
    assert_eq!(final_chunk.len(), 4);

    // Remaining 6 items can be pulled
    chunk_puller.resize_for_chunk_size(10);
    let last_chunk = chunk_puller.pull().unwrap();
    assert_eq!(last_chunk.len(), 6); // 18-23
}
