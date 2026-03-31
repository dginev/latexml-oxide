//! pgfmathcalc.code.tex — PGF math calculation macros
//! Perl: pgfmathcalc.code.tex.ltxml (34 lines)
//!
//! Loads the raw TeX code and provides \pgfmathsetmacro in Rust.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L20: Load pgf's TeX code for math calc first
  InputDefinitions!("pgfmathcalc.code", extension => Some(Cow::Borrowed("tex")), noltxml => true);

  // Perl L24-32: \pgfmathsetmacro — evaluates expression and defines macro
  // Evaluates #2 via pgfmathparse, then \edef's #1 to the result.
  // Perl wraps in begingroup/endgroup to isolate side effects, then
  // bypasses \pgfmath@smuggleone by using Perl-level DefMacroI.
  DefPrimitive!("\\pgfmathsetmacro{}{}", sub[(cs, expression)] {
    let expr_str = do_expand(expression)?.to_string();
    bgroup();
    let result = crate::package::pgfmath_code_tex::pgfmathparse_eval(&expr_str);
    egroup()?;
    let result_tokens = mouth::tokenize_internal(&result);
    def_macro(cs.unlist().into_iter().next().unwrap_or(T_CS!("\\pgfmathresult")),
      None, result_tokens, None)?;
    Ok(Vec::new())
  });
});
