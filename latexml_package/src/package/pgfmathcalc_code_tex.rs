//! pgfmathcalc.code.tex — PGF math calculation macros
//! Perl: pgfmathcalc.code.tex.ltxml (34 lines)
//!
//! Loads the raw TeX code and provides \pgfmathsetmacro in Rust.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L20: Load pgf's TeX code for math calc first
  InputDefinitions!("pgfmathcalc.code", extension => Some(Cow::Borrowed("tex")), noltxml => true);

  // Perl L24-32: \pgfmathsetmacro — evaluates expression and defines macro.
  // Perl's DefMacroI call explicitly passes `scope => 'local'`, so the
  // new CS vanishes when the enclosing group closes — essential for
  // tikz/pgf loops that reuse the same CS (e.g. \foreach \n …). The
  // Rust port had dropped the scope, leaking redefinitions upward past
  // the begingroup/endgroup wrapper. Restore Scope::Local to match Perl.
  DefPrimitive!("\\pgfmathsetmacro{}{}", sub[(cs, expression)] {
    let expr_str = do_expand(expression)?.to_string();
    bgroup();
    let result = crate::package::pgfmath_code_tex::pgfmathparse_eval(&expr_str);
    egroup()?;
    let result_tokens = mouth::tokenize_internal(&result);
    def_macro(
      cs.unlist().into_iter().next().unwrap_or_else(|| T_CS!("\\pgfmathresult")),
      None,
      result_tokens,
      Some(ExpandableOptions { scope: Some(Scope::Local), ..Default::default() }))?;
    Ok(Vec::new())
  });
});
