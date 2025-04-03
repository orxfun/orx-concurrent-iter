use crate::implementations::ConIterEmpty;

pub fn empty<T: Send + Sync>() -> ConIterEmpty<T> {
    Default::default()
}
