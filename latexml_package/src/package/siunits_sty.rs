use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: SIunits.sty.ltxml
  // Apparently siunitx is a revision and extension of SIunits
  // Not quite backwards compatible, but worth a try...
  RequirePackage!("siunitx");

  // Apparently similar, but expects the numbers to be already formatted?
  // (things like \times, ^, etc appear)
  Let!("\\unit", "\\SI");

  RawTeX!("\\sisetup{parse-numbers = false, input-product = \\times,}");

  DefMacro!("\\squaren{}", "{#1}^{2}");

  // NOTE: do NOT `six_enable_unit_macros(true)` here. The real SIunits package
  // defines short single-letter unit macros (`\m`=metre, `\s`=second, `\g`,
  // `\A`, …), but Perl's `SIunits.sty.ltxml` is a thin shim — it only
  // `RequirePackage('siunitx')` + `\squaren`, defining NONE of those macros
  // (siunitx uses `\si{\metre}` syntax instead). Enabling them here was a
  // Rust-only divergence: defining `\m` clobbers the extremely common user
  // macro `\newcommand{\m}[1]{\(#1\)}` (a LaTeX `\newcommand` of an
  // already-defined CS is silently ignored), so every `\m{…}` rendered as the
  // metre symbol in TEXT mode → 753× `^`/`_` "can only appear in math mode"
  // (witness 1509.04521, `\usepackage[squaren]{SIunits}` + `\newcommand{\m}`).
  // Matching Perl (which leaves `\m` for the document) fixes it; SIunits docs
  // that genuinely use `\m`/`\metre` standalone already fail in Perl too.
});
