//! Dynamically-typed values stored in EDN
//!
//! ## Implementations
//! -  [`DataTrait`] blanket-implemented for all types that implement [`Debug`], [`Display`], [`Clone`], [`PartialEq`], [`Eq`], [`PartialOrd`], [`Ord`] & [`Hash`]

use alloc::boxed::Box;
use core::{
  any::Any,
  fmt,
  hash::{Hash, Hasher},
};

/// Dyn-compatible trait to store dynamically-typed values present in [`Edn::Data`][Data]
///
/// [Data]: crate::edn::Edn::Data
pub trait DataTrait: Any + fmt::Debug + fmt::Display {
  fn clone_(&self) -> Box<dyn DataTrait>;
  fn eq_(&self, other: &dyn DataTrait) -> bool;
  fn cmp_(&self, other: &dyn DataTrait) -> core::cmp::Ordering;
  fn hash_(&self, state: &mut dyn Hasher);
  fn partial_cmp_(&self, other: &dyn DataTrait) -> Option<core::cmp::Ordering> {
    Some(self.cmp_(other))
  }
}

impl<T> DataTrait for T
where
  T: fmt::Debug + fmt::Display + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + 'static,
{
  fn clone_(&self) -> Box<dyn DataTrait> {
    Box::new(self.clone())
  }

  fn eq_(&self, other: &dyn DataTrait) -> bool {
    let other: &dyn Any = other;
    other.downcast_ref().is_some_and(|other| self.eq(other))
  }

  fn partial_cmp_(&self, other: &dyn DataTrait) -> Option<core::cmp::Ordering> {
    let other: &dyn Any = other;
    other.downcast_ref().and_then(|other| self.partial_cmp(other))
  }

  fn cmp_(&self, other: &dyn DataTrait) -> core::cmp::Ordering {
    let other: &dyn Any = other;
    let other: &T = other.downcast_ref().expect("Expected same lhs & rhs erased-types");
    self.cmp(other)
  }

  fn hash_(&self, state: &mut dyn Hasher) {
    /// Hasher that wraps reference `&'a mut dyn Hasher`
    struct Adapter<'a> {
      state: &'a mut dyn Hasher,
    }
    impl Hasher for Adapter<'_> {
      fn finish(&self) -> u64 {
        self.state.finish()
      }
      fn write(&mut self, bytes: &[u8]) {
        self.state.write(bytes);
      }
    }

    self.hash(&mut Adapter { state });
  }
}

/// Pointer to a dynamically-typed value, used in [`Edn::Data`](crate::edn::Edn::Data)
#[derive(Debug)]
pub struct Datum(Box<dyn DataTrait>);

impl fmt::Display for Datum {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl Clone for Datum {
  fn clone(&self) -> Self {
    Self(self.0.clone_())
  }
}

impl PartialEq for Datum {
  fn eq(&self, other: &Self) -> bool {
    Any::type_id(&self.0) == Any::type_id(&other.0) && self.0.eq_(&*other.0)
  }
}

impl Eq for Datum {}

impl PartialOrd for Datum {
  #[expect(clippy::non_canonical_partial_ord_impl)]
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    self.0.partial_cmp_(&*other.0)
  }
}

impl Ord for Datum {
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    self.0.cmp_(&*other.0)
  }
}

impl Hash for Datum {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.hash_(state);
  }
}

impl Datum {
  /// Requires that `T` implement `Debug`, `Display`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord` & `Hash`
  pub fn new<T: DataTrait>(t: T) -> Self {
    Self(Box::new(t))
  }

  /// Requires that `T` implement `Debug`, `Display`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord` & `Hash`
  pub fn from_boxed<T: DataTrait>(t: Box<T>) -> Self {
    Self(t)
  }

  /// Downcast the datum to an expected concrete-type `T`
  ///
  /// # Errors
  ///
  /// Returns the original data-pointer `Box<dyn Any>` in case the concrete type didn't correspond
  pub fn downcast<T: DataTrait + 'static>(self) -> Result<Box<T>, Box<dyn Any>> {
    let o: Box<dyn Any> = self.0;
    o.downcast()
  }
}
