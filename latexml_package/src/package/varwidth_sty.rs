use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: varwidth.sty.ltxml
  // Don't bother with distinction between {varwidth} and {minipage}
  DefMacro!("\\varwidth", "\\minipage");
  DefMacro!("\\endvarwidth", "\\endminipage");
  Let!("\\narrowragged", "\\raggedright");
  // varwidth.sty:308-314 — the `V{width}` column (a top-aligned varwidth
  // cell, `\\` re-let to the row break) when array's `\newcolumntype`
  // exists; Perl's varwidth.sty.ltxml omits it, so `{lccV{\linewidth}l}`
  // lost a column and every 5th cell was an "Extra alignment tab" (numerica
  // ×28). Guard: `perfect_kernel_batch54::varwidth_v_column_is_defined`.
  RawTeX!(
    r"\@ifundefined{newcolumntype}{}{\@ifundefined{NC@rewrite@V}{\newcolumntype{V}[1]{>{\begin{varwidth}[t]{#1}\narrowragged\let\\\tabularnewline}l<{\@finalstrut\@arstrutbox\end{varwidth}}}}{}}"
  );
});
