//! Generic support layer for any LaTeXML package binding file.
//!
//! Re-exports `latexml_engine::prelude::*` (the shared macro layer +
//! latexml_core re-exports) and adds package-specific symbols that are
//! only meaningful at the binding layer (`crate::package::*`,
//! `GetKeyVal` constructor helper).

pub use latexml_engine::prelude::*;

// Package-level API. Engine code does not need this; only the
// `package/*.rs` files (and downstream binaries / contrib) do.
pub use crate::package::*;

// Functions callable from constructor templates via &GetKeyVal(#1,key) syntax.
// Returns Option<Digested> to be compatible with both attribute (to_attribute/to_string)
// and body absorption (Into<Option<Digested>>) contexts in constructor templates.
#[allow(non_snake_case)]
pub fn GetKeyVal(keyval_opt: &Option<Digested>, key: &str) -> Option<Digested> {
  match keyval_opt {
    Some(digested) => match digested.data() {
      DigestedData::KeyVals(keyval) => keyval.get_value_digested(key).cloned(),
      _ => None,
    },
    _ => None,
  }
}
