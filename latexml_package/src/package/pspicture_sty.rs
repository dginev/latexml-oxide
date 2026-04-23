use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl pspicture.sty.ltxml L18-31 declares \Line / \Vector / \Curve with
  // proper signatures consuming a trailing `(x,y)` Pair (plus a `{Float}`
  // for \Curve). The full DefConstructors emit <ltx:line> / <ltx:bezier>
  // via picProperties / picScale helpers, which aren't ported to Rust.
  //
  // Until those picture helpers land, at least consume the Pair argument
  // so author code like `\Line(10,20)` doesn't leak "(10,20)" into the
  // surrounding text after expansion.
  DefMacro!("\\Line Pair", "");
  DefMacro!("\\Vector Pair", "");
  DefMacro!("\\Curve Pair {}", "");
});
