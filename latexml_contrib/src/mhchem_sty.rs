//! mhchem.sty — chemical formula typesetting.
//!
//! TODO(strict-perl-parity): once `latexml_engine` can faithfully
//! handle the expl3 / xparse / chemgreek raw-load chain (currently
//! the gaps are around `\group_begin:` non-boxing-frame handling
//! and l3regex/l3tl-analysis register access during the chemgreek
//! load triggered by the first `\ce{...}`), DELETE this binding so
//! that `\usepackage{mhchem}` raw-loads the actual TL `mhchem.sty`,
//! matching Perl LaTeXML's behavior (Perl has no `mhchem.sty.ltxml`).
//! Driver paper: arXiv:1806.06448 (3 errors → 0 errors with this
//! stub; full chemistry rendering needs the engine fix).
//!
//! Perl LaTeXML has no `mhchem.sty.ltxml` and raw-loads the actual
//! TL `mhchem.sty` (which `\RequirePackage{chemgreek}` →
//! `\RequirePackage{xparse}` → heavy expl3 machinery). Perl's expl3
//! emulation is mature enough that this works.
//!
//! Rust's expl3 emulation has gaps (e.g. `\group_begin:` non-boxing
//! frame handling, `\l__tl_analysis_*_int` register access in
//! l3regex/l3tl-analysis), so the chemgreek raw-load triggered by the
//! first `\ce{...}` invocation leaves the gullet in an unbalanced state
//! (open `\iffalse`, unmatched `{` at end-of-input).
//!
//! Until the expl3 cluster is fixed, this binding intercepts the
//! mhchem load and provides a minimal stub: `\ce{...}` typesets its
//! argument as roman text, no chemistry layout. This is a documented
//! divergence from Perl LaTeXML — the full chemistry rendering needs
//! a real port. Driver paper: 1806.06448 (3 errors → 0 errors).
//!
//! Stubs cover the public mhchem v3/v4 surface most papers actually
//! use: `\ce`, `\cee`, `\cf`, plus `\mhchemoptions` (no-op).

use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl LaTeXML auto-scans mhchem.sty for `\RequirePackage` calls
  // and brings in ifthen, calc, twoopt, amsmath, keyval, graphics, pgf,
  // tikz as transitive deps. Since this Rust stub intercepts the load
  // (so the raw RequirePackage chain never fires), papers that rely on
  // those deps via mhchem alone hit undefined-CS errors. Pull in the
  // ones most commonly needed: amsmath (for \boldsymbol, \eqref,
  // \text, align*, etc.) and graphicx (for figure handling). Witness:
  // 1311.6762 (stage 15 RUST-REGRESSION) — paper loads mhchem but
  // not amsmath, then uses `\boldsymbol` / `\eqref`. Perl's auto-dep
  // scan loads amsmath → 0 errors; Rust stub didn't → 2 errors.
  RequirePackage!("amsmath");
  RequirePackage!("graphicx");

  // Accept both v3 and v4: the package option is `version=N` — handled
  // at \usepackage time but irrelevant to our stub.
  DefMacro!("\\mhchemoptions RequiredKeyVals", "");

  // \ce{<formula>} — chemistry mode. Real mhchem renders subscripts,
  // charges, arrows, etc. Papers invoke \ce{H_2O} / \ce{N_2} both in
  // math context (equation*) AND in text context (paragraphs).
  // \ensuremath wraps body in math mode if not already in math, so
  // `_`/`^` parse as scripts in both contexts. Loses roman-text
  // rendering of plain text chemistry, but avoids cascading errors.
  //
  // Strip embedded `$` toggles from the body before re-entering math:
  // mhchem v3 papers commonly write `\ce{Cs$_x$MA$_{1-x}$PbI3}` where
  // the `$` pairs are mhchem's own subscript-grouping hint, NOT real
  // math toggles. Without stripping, `\ensuremath{...$_x$...}` re-toggles
  // out of math at the first `$`, leaving `_x` in text mode — which
  // errors with "Script _ can only appear in math mode".
  // Witnesses: 1908.05236 (\ce{MAPb(I_{1-x}Br_x)3}), 0907.1390 (\ce{N_2}).
  fn strip_math_toggles(arg: &Tokens) -> Tokens {
    let stripped: Vec<Token> = arg.unlist_ref().iter().copied()
      .filter(|t| t.get_catcode() != Catcode::MATH)
      .collect();
    Tokens::new(stripped)
  }
  DefMacro!("\\ce{}", sub[(body)] {
    let stripped = strip_math_toggles(&body);
    let mut result = vec![T_CS!("\\ensuremath"), T_BEGIN!()];
    result.extend(stripped.unlist());
    result.push(T_END!());
    Ok(Tokens::new(result))
  });
  DefMacro!("\\cee{}", sub[(body)] {
    let stripped = strip_math_toggles(&body);
    let mut result = vec![T_CS!("\\ensuremath"), T_BEGIN!()];
    result.extend(stripped.unlist());
    result.push(T_END!());
    Ok(Tokens::new(result))
  });
  DefMacro!("\\cf{}", sub[(body)] {
    let stripped = strip_math_toggles(&body);
    let mut result = vec![T_CS!("\\ensuremath"), T_BEGIN!()];
    result.extend(stripped.unlist());
    result.push(T_END!());
    Ok(Tokens::new(result))
  });

  // \arrow / \chemarrow — used inside \ce arguments. Stub as small text
  // arrow so a `\ce{A \arrow B}` doesn't error if it leaks out.
  DefMacro!("\\chemarrow", "\\rightarrow");
});
