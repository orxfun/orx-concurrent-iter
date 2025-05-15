pub trait RawVectors: Default {
    type Element;

    fn vec_lengths(&self) -> impl Iterator<Item = usize>;
}
