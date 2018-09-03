pub use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::{Arc, Mutex};

lazy_static! {
  static ref CROW_ARENA: Mutex<HashMap<String, Arc<String>>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Clone)]
pub enum Crow {
  /// Borrowed data.
  Borrowed(&'static str),
  /// Shared reference-counted data.
  Shared(Arc<String>),
}

impl Borrow<str> for Crow {
  fn borrow(&self) -> &str { &**self }
}

impl Deref for Crow {
  type Target = str;
  fn deref(&self) -> &str {
    match self {
      Crow::Borrowed(borrowed) => borrowed,
      Crow::Shared(ref rc) => (**rc).borrow(),
    }
  }
}

impl Display for Crow {
  fn fmt(&self, f: &mut Formatter) -> Result {
    match *self {
      Crow::Borrowed(ref b) => Display::fmt(b, f),
      Crow::Shared(ref o) => Display::fmt(o, f),
    }
  }
}

impl PartialEq<Crow> for Crow {
  #[inline]
  fn eq(&self, other: &Crow) -> bool { PartialEq::eq(&**self, &**other) }
}

impl Hash for Crow {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) { Hash::hash(&**self, state) }
}

impl Crow {
  /// Memory arena setter
  pub fn into_arena<T: ToString>(text: T) -> Arc<String> {
    let text: String = text.to_string();
    let mut arena = CROW_ARENA.lock().unwrap();
    // .entry() requires a key clone, so...
    if let Some(value) = arena.get(&text) {
      return value.clone();
    }
    // new value, insert.
    let new_v: Arc<String> = Arc::new(text.to_string());
    arena.insert(text, new_v.clone());
    new_v
  }
}

impl<'a> From<&'a str> for Crow {
  fn from(s: &'a str) -> Crow { Crow::Shared(Crow::into_arena(s)) }
}
