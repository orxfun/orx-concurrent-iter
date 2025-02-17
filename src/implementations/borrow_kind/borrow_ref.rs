use super::borrow_kind_trait::BorrowKind;

pub struct BorrowRef;

impl<'a, T> BorrowKind<'a, T> for BorrowRef {}
