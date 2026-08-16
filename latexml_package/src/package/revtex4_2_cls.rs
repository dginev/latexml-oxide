use crate::prelude::*;

// REVTeX 4-2 document class.
//
// Perl has no `revtex4-2.cls.ltxml`: it version-falls `revtex4-2` to the generic
// `revtex.cls`, which preloads natbib WITH the `numbers` option, so APS journals
// (`pre`, `prl`, `prb`, …) get NUMERIC citations (`[1]`, `[2]`, numbered list) —
// the style the pdflatex-built PDF uses. Oxide keeps the richer revtex4 routing
// instead of that ancient revtex3 fallback: `revtex4_cls` declares the revtex4
// options `revtex.cls` lacks (`pre`, `superscriptaddress`, groupedaddress, …).
//
// To reproduce Perl's numeric default without losing that option coverage, we
// set a flag and delegate to `revtex4_cls`, which — seeing the flag — preloads
// `natbib[numbers]` before revtex4_support pulls natbib in with no options.
// natbib loads once, so the document's own later `\usepackage{natbib}` is a
// no-op and the numeric style sticks. Bare `revtex4` / `revtex4-1` leave the
// flag unset and keep natbib's author-year default (Perl parity for those class
// names). Witness arXiv 2606.09494 (html_feedback #6609).
pub fn load_definitions() -> Result<()> {
  assign_value("revtex_cite_numbers", true, Some(Scope::Global));
  revtex4_cls::load_definitions()
}
