use std::{
  any::type_name,
  collections::hash_map::{Entry, IntoIter, Iter, IterMut, Keys},
  fmt,
  iter::IntoIterator,
  ops::Index,
};

use rustc_hash::FxHashMap as HashMap;
use string_interner::{Symbol, symbol::SymbolU32};

use crate::common::arena;

pub type SymStr = SymbolU32;

// TODO: Are we heading in the right direction with this interface...
// is there performance overhead from the extra wrap? It seems borderline usable...

/// A convenience abstraction over a String-keyed HashMap
///
/// typically used for `HashMap<String,Stored>` states.
/// The goal is to support both a string interface, as well as the interned tickets interface,
/// while avoiding String allocations internally.
#[derive(Clone)]
pub struct SymHashMap<T>(pub HashMap<SymStr, T>);

impl<T> Default for SymHashMap<T> {
  fn default() -> Self { SymHashMap(HashMap::default()) }
}

impl<T> SymHashMap<T> {
  #[inline]
  pub fn len(&self) -> usize { self.0.len() }
  #[inline]
  pub fn is_empty(&self) -> bool { self.0.is_empty() }
  #[inline]
  pub fn get(&self, key: &str) -> Option<&T> { self.0.get(&arena::pin(key)) }
  #[inline]
  pub fn get_sym(&self, key: SymStr) -> Option<&T> { self.0.get(&key) }
  #[inline]
  pub fn get_mut(&mut self, key: &str) -> Option<&mut T> { self.0.get_mut(&arena::pin(key)) }
  #[inline]
  pub fn get_mut_sym(&mut self, key: SymStr) -> Option<&mut T> { self.0.get_mut(&key) }
  #[inline]
  pub fn contains_key(&self, key: &str) -> bool { self.0.contains_key(&arena::pin(key)) }
  #[inline]
  pub fn contains_key_sym(&self, key: &SymStr) -> bool { self.0.contains_key(key) }
  #[inline]
  pub fn insert(&mut self, key: &str, value: T) { self.0.insert(arena::pin(key), value); }
  #[inline]
  pub fn insert_sym(&mut self, key: SymStr, value: T) { self.0.insert(key, value); }
  #[inline]
  pub fn remove(&mut self, key: &str) { self.0.remove(&arena::pin(key)); }
  #[inline]
  pub fn remove_sym(&mut self, key: SymStr) { self.0.remove(&key); }
  #[inline]
  pub fn keys(&self) -> Keys<'_, SymStr, T> { self.0.keys() }
  #[inline]
  pub fn entry(&mut self, key: &str) -> Entry<'_, SymStr, T> { self.0.entry(arena::pin(key)) }
  #[inline]
  pub fn entry_sym(&mut self, key: SymStr) -> Entry<'_, SymStr, T> { self.0.entry(key) }
  #[inline]
  pub fn iter(&self) -> Iter<'_, SymStr, T> { self.0.iter() }
}

impl<'a, T> IntoIterator for &'a SymHashMap<T> {
  type Item = (&'a SymStr, &'a T);
  type IntoIter = Iter<'a, SymStr, T>;

  #[inline]
  fn into_iter(self) -> Iter<'a, SymStr, T> { self.0.iter() }
}

impl<'a, T> IntoIterator for &'a mut SymHashMap<T> {
  type Item = (&'a SymStr, &'a mut T);
  type IntoIter = IterMut<'a, SymStr, T>;

  #[inline]
  fn into_iter(self) -> IterMut<'a, SymStr, T> { self.0.iter_mut() }
}
impl<T> IntoIterator for SymHashMap<T> {
  type Item = (SymStr, T);
  type IntoIter = IntoIter<SymStr, T>;
  #[inline]
  fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<T: fmt::Debug> fmt::Debug for SymHashMap<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "SymHashMap[")?;
    let mut init = true;
    for (k, v) in self {
      if init {
        init = false;
      } else {
        write!(f, ", ")?;
      }
      arena::with(*k, |key| write!(f, "{key}"))?;
      // very temporary hack to get the full trace
      if type_name::<T>() == "string_interner::symbol::SymbolU32" {
        let symstr = format!("{:?}", v);
        // "SymbolU32 { value: 28104 }"
        let mut symiter = symstr.split(' ');
        symiter.next();
        symiter.next();
        symiter.next();
        let sym_v_str = symiter.next().unwrap();
        let sym_val = sym_v_str.parse::<usize>().unwrap();
        let sym = Symbol::try_from_usize(sym_val - 1).unwrap();
        let vstr = arena::to_string(sym);
        write!(f, ": {vstr}")?;
      } else {
        write!(f, ": [{:?}]", v)?;
      }
      // write!(f,": {:?}",v)?;
    }
    write!(f, "]")
  }
}

impl<T> Index<&SymStr> for SymHashMap<T> {
  type Output = T;
  /// Returns a reference to the value corresponding to the supplied key.
  ///
  /// # Panics
  ///
  /// Panics if the key is not present in the `HashMap`.
  #[inline]
  fn index(&self, key: &SymStr) -> &T { &self.0[key] }
}
