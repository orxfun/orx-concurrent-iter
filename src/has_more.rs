/// Returns whether or not the concurrent iterator has more elements to yield.
/// Response is guaranteed to be true.
///
/// * `Yes(n)`: the iterator will certainly yield `n` more elements.
/// * `Maybe`: the iterator might or might not yield any more element. A concurrent iterator created from an iterator with a non-trivial filter could be considered as an example. There certainly exists elements to evaluate, but not guaranteed to return one.
/// * `No`: the iterator has terminated, and will certainly not yield any more elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HasMore {
    /// The iterator will certainly yield `n` more elements.
    Yes(usize),
    /// The iterator might or might not yield any more element. A concurrent iterator created from an iterator with a non-trivial filter could be considered as an example. There certainly exists elements to evaluate, but not guaranteed to return one.
    Maybe,
    /// The iterator has terminated, and will certainly not yield any more elements.
    No,
}
