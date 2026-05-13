use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: kvoptions.sty.ltxml
  InputDefinitions!("kvoptions", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // kvoptions.sty defines `\ProcessLocalKeyvalOptions` to gate on
  // `\@currext == \@pkgextension` — i.e. "we're in a package context".
  // When hyperref's backend-specific `.def` files (hpdftex.def,
  // hluatex.def, ...) raw-load and transitively `\RequirePackage{
  // rerunfilecheck}`, rerunfilecheck.sty calls `\ProcessLocalKeyvalOptions*`
  // and the guard fires `\PackageError{kvoptions}{\ProcessLocalKeyvalOptions
  // is intended for packages only}`. This becomes our
  // `Error:latex:\GenericError` event.
  //
  // Perl LaTeXML avoids the chain entirely: its hyperref binding is a
  // hand-ported `.ltxml` that doesn't raw-load the backend `.def` files,
  // so rerunfilecheck is never required. Rust's hyperref binding
  // currently raw-loads the chain.
  //
  // Rather than rewriting hyperref-backend-load behaviour, override the
  // CS so the keyval-options processing is a no-op (matches our paradigm
  // — see WISDOM #50, feedback_size_layout_errors_moot.md). This is the
  // engine-level analog of what `\@onlypreamble` does for unrelated
  // typesetting state: kvoptions's PDF-backend keyval state doesn't
  // affect XML/HTML output, so dropping it is lossless for us.
  //
  // Witnesses: arXiv:cond-mat/9611206, math/9904040, math/9904041.
  DefMacro!("\\ProcessLocalKeyvalOptions OptionalMatch:*", "");
});
