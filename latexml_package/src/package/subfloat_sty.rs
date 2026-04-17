//! subfloat.sty — Subfigure/subtable container environments
//! Perl: subfloat.sty.ltxml — 100 lines
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Convenience macros — Perl L25-28
  DefMacro!("\\subfiguresbegin", "\\begin{subfigures}");
  DefMacro!("\\subfiguresend",   "\\end{subfigures}");
  DefMacro!("\\subtablesbegin",  "\\begin{subtables}");
  DefMacro!("\\subtablesend",    "\\end{subtables}");

  // Counters — Perl L30-32
  NewCounter!("subfloatfigure", "figure", idprefix => "sf", idwithin => "figure");
  NewCounter!("subfloattable",  "table",  idprefix => "st", idwithin => "table");
  // Perl sets \themainfigure/\themaintable in {subfigures}/{subtables} beforeDigest;
  // we provide global Let aliases so they resolve outside that scope too.
  Let!("\\themainfigure", "\\thefigure");
  Let!("\\themaintable",  "\\thetable");
  DefMacro!("\\thesubfloatfigure", "\\themainfigure\\alph{subfloatfigure}");
  DefMacro!("\\thesubfloattable",  "\\themaintable\\alph{subfloattable}");
  DefMacro!("\\subfloatfigurename", "Figure");
  DefMacro!("\\subfloattablename",  "Table");
  Let!("\\ext@subfloatfigure", "\\ext@figure");
  Let!("\\ext@subfloattable",  "\\ext@table");
  DefMacro!("\\fnum@subfigure", "(\\thesubfigure)");
  DefMacro!("\\fnum@subtable",  "(\\thesubtable)");

  // Container environments — Perl L38-85
  // {subfigures} redefines {figure} internally to use subfloatfigure counter
  DefEnvironment!("{subfigures}",
    "<ltx:figure xml:id='#id' inlist='lof'>#tags#body</ltx:figure>",
    mode => "internal_vertical"
  );
  DefEnvironment!("{subtables}",
    "<ltx:table xml:id='#id' inlist='lot'>#tags#body</ltx:table>",
    mode => "internal_vertical"
  );
});
