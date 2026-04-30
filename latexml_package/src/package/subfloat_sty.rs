//! subfloat.sty — Subfigure/subtable container environments
//! Perl: subfloat.sty.ltxml — 100 lines
use crate::engine::latex_constructs::{after_float, before_float};
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

  // Container environments — Perl L40-67 / L70-97. The previous Rust port
  // omitted the per-env `before_digest`/`after_digest` hooks entirely:
  // Perl wires `beforeFloat('figure')` (steps the figure counter,
  // pushes float scope) plus `afterFloat($whatsit)` (pops scope, sets
  // refnum), so a `\begin{subfigures}` properly registers as a float and
  // its caption can resolve. Also `inlist='lof'` was hardcoded — Perl
  // uses `inlist='#inlist'` driven by RefStepCounter.
  //
  // BLOCKER: Perl ALSO redefines {figure}/{figure*} (and table/table*)
  // INSIDE the before_digest closure to switch them to the
  // subfloat<figure|table> counter and drop the inlist attribute. That
  // requires a runtime def_environment from inside a before_digest
  // closure (enumitem/float pattern). Deferred — the simple case
  // (single-level subfigure with manual \subfigure marks) still works
  // with the basic before/after_float hooks added here.
  DefEnvironment!("{subfigures}",
    "<ltx:figure xml:id='#id' inlist='#inlist'>#tags#body</ltx:figure>",
    before_digest => {
      Let!("\\themainfigure", "\\thefigure");
      before_float("figure", None);
    },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );
  DefEnvironment!("{subtables}",
    "<ltx:table xml:id='#id' inlist='#inlist'>#tags#body</ltx:table>",
    before_digest => {
      Let!("\\themaintable", "\\thetable");
      before_float("table", None);
    },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );
});
