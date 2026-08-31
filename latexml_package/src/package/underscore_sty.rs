use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: underscore.sty.ltxml
  // Don't really need to change \_, but do need to make _ work in text!
  DefMacro!(T_ACTIVE!('_'), None, "\\ifmmode\\sb\\else\\textunderscore\\fi");
  // underscore.sty L38: the public breakable text underscore. Perl's binding
  // omits it, but raw third-party code calls it DIRECTLY — l3doc.cls L694
  // rewrites active `_` into `\BreakableUnderscore` calls, so every
  // l3doc-built manual errored `undefined` (6 TL doc bundles, incl.
  // l3kernel's own). Break behavior is presentation; the glyph is the
  // content.
  DefMacro!("\\BreakableUnderscore", "\\textunderscore");
  at_begin_document(TokenizeInternal!(r"\catcode`_\active"))?;
});
