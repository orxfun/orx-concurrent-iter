use super::borrow_kind_trait::BorrowKind;

pub struct BorrowMut;

impl<'a, T> BorrowKind<'a, T> for BorrowMut {}
