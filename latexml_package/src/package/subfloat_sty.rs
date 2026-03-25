use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\subfiguresbegin", "\\begin{subfigures}");
  DefMacro!("\\subfiguresend",   "\\end{subfigures}");
  DefMacro!("\\subtablesbegin",  "\\begin{subtables}");
  DefMacro!("\\subtablesend",    "\\end{subtables}");
  NewCounter!("subfloatfigure", "figure", idprefix => "sf", idwithin => "figure");
  NewCounter!("subfloattable",  "table",  idprefix => "st", idwithin => "table");
  DefMacro!("\\thesubfloatfigure", "\\themainfigure\\alph{subfloatfigure}");
  DefMacro!("\\thesubfloattable",  "\\themaintable\\alph{subfloattable}");
  DefMacro!("\\subfloatfigurename", "Figure");
  DefMacro!("\\subfloattablename",  "Table");
  Let!("\\ext@subfloatfigure", "\\ext@figure");
  Let!("\\ext@subfloattable",  "\\ext@table");
  DefMacro!("\\fnum@subfigure", "(\\thesubfigure)");
  DefMacro!("\\fnum@subtable",  "(\\thesubtable)");
});
