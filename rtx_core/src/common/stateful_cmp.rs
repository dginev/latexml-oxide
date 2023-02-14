use crate::common::State;

pub trait StatefulEq<Rhs: ?Sized = Self> {
  /// This method tests for `self` and `other` values to be equal, and is used
  /// by `==`.
  #[must_use]
  fn eq(&self, other: &Rhs, state: &State) -> bool;

  /// This method tests for `!=`. The default implementation is almost always
  /// sufficient, and should not be overridden without very good reason.
  #[inline]
  #[must_use]
  fn ne(&self, other: &Rhs, state: &State) -> bool {
    !self.eq(other, state)
  }
}