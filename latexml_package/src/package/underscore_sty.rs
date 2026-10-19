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
  // underscore.sty:26-31 `\AtBeginDocument{… \catcode\string`\_\string=\active …}`: registered in the kernel's
  // begin-document hook in load order, so a package loaded later that sets `_` again at the document's start wins, as in
  // pdflatex — mathstyle.sty:233-236 makes it other (12). The binding's `at_begin_document` list (Perl `AtBeginDocument`)
  // ran after every `\AtBeginDocument` and left `_` active, a subscript in text (2507.09194).
  RawTeX!(r"\AtBeginDocument{\catcode`\_\active}");
});
