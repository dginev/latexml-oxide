//! chemformula.sty (chemical formulas, expl3) — raw-loaded.
//!
//! The binding used to alias `\ch`/`\chcpd` to mhchem's `\ce` and stub the
//! `chemformula/*` keys. mhchem's parser rejects chemformula's own syntax —
//! `"text"` literals, `\ch{"H_2O"}`-style groups — with "Assertion failed:
//! Unexpected input character" (chemformula-manual ×50, endiagram, modiagram),
//! and `\chemsetup{chemformula/format=…}` reported an unknown key. Perl has no
//! chemformula binding and raw-loads the real package too. Batch 56az retires
//! the stub: the real `\ch`, `\chcpd`, `\chemformula_*` API (used by
//! chemmacros.sty:1358-1366) and keys come from chemformula.sty itself.
use std::borrow::Cow;

use latexml_package::prelude::*;

LoadDefinitions!({
  // chemformula is `\ProvidesExplPackage` + `\ProcessKeysPackageOptions`
  // (chemformula.sty:41, :481): l3keys2e/xparse must precede the raw load
  // (witness 2504.13749). chemformula.sty:29 requires tikz, amsmath, xfrac,
  // nicefrac — tikz pulls xcolor, which a document's
  // `\PassOptionsToPackage{table}{xcolor}` relies on (witness 1809.04023,
  // revtex4-1 + `\rowcolors`); xfrac gives the document `\sfrac` (witness
  // 2006.07679).
  RequirePackage!("l3keys2e");
  RequirePackage!("xparse");
  RequirePackage!("tikz");
  RequirePackage!("amsmath");
  RequirePackage!("xfrac");
  RequirePackage!("nicefrac");
  InputDefinitions!("chemformula", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
