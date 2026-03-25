use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // The Perl source defines \xspace as a macro sub that reads the next
  // token and checks if it's in a set of tokens that should NOT get
  // extra space. If the next token is not in the set, a space is inserted.
  // We approximate with a no-op since the sub-based implementation
  // requires gullet readToken which is complex in the compile-time macro system.
  DefMacro!("\\xspace", None);
});
