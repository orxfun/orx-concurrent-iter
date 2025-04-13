use crate::implementations::ConIterEmpty;

/// Creates an empty concurrent iterator which does not yield any elements.
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let con_iter = iter::empty::<String>();
/// assert_eq!(con_iter.next(), None);
///
/// // or
///
/// let con_iter = implementations::ConIterEmpty::<String>::new();
/// assert_eq!(con_iter.next(), None);
/// ```
pub fn empty<T: Send + Sync>() -> ConIterEmpty<T> {
    Default::default()
}
