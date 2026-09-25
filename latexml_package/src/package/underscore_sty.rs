use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: underscore.sty.ltxml
  // Don't really need to change \_, but do need to make _ work in text!
  // The kernel's first aid replaces this right after the package loads
  // (latex2e-first-aid-for-external-files.ltx:176-177 → the underscore-ltx
  // binding in latexml_contrib).
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
