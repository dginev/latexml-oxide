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

// Native equivalents of Perl `AtBeginDocument` / `AtEndDocument`
// (`LaTeXML/Package.pm:2798-2826`). Append tokens to the queue consumed by
// `\begin{document}` / `\end{document}` (latex_constructs.rs).
//
// Bindings should prefer these over `RawTeX!(r"\AtBeginDocument{...}")` so
// the queue is populated directly without round-tripping through the
// `\AtBeginDocument` macro (which expl3 redefines to route through the
// L3 hook system).
pub fn at_begin_document<T: Into<Stored>>(operations: T) -> Result<()> {
  state::push_value("@at@begin@document", operations)
}

pub fn at_end_document<T: Into<Stored>>(operations: T) -> Result<()> {
  state::push_value("@at@end@document", operations)
}
