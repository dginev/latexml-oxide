use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "arydshln.sty",
    "arydshln.sty is only minimally stubbed and will not be interpreted raw."
  );
  // TODO (deferred, still open): extend the Alignment machinery with a dashed
  // bottom-border directive; `\hdashline` draws a solid `border="t"` until then.
  // The dashed rules take an optional `[<dash>/<gap>]` (arydshln.sty:432-438
  // `\adl@hdashline` → `\@ifnextchar[`, :459-462 `\cdashline#1` likewise,
  // :482-483 `\firsthdashline`/`\lasthdashline`); drawn as plain rules here,
  // so the spec is read and dropped. As `\let`s of `\hline`/`\cline` (Perl
  // arydshln.sty.ltxml:18-19, KNOWN_PERL_ERRORS #291) the `[2pt/2pt]` was left
  // as text in the next row's first cell (arXiv 2605.19920).
  DefMacro!("\\hdashline[]", "\\hline");
  DefMacro!("\\cdashline{}[]", "\\cline{#1}");
  // ar5iv-bindings/bindings/arydshln.sty.ltxml L21-24: ':' column type adds a
  // dashed right-border marker via the between-column slot. The \vrule gets
  // decorated by \@ADDCLASS{ltx_border_r_dashed}\relax so CSS renders a
  // dashed vertical rule.
  DefColumnType!(":", {
    with_current_build_template(|template_opt| {
      template_opt
        .unwrap()
        .add_between_column(dashed_rule_between_columns());
    });
  });
  // `;{<dash>/<gap>}` is `:` with its own dash pattern (arydshln.sty:198/236
  // map `;` to `\adl@argarraydashrule`, :270-273, whose `\adl@classvfordash`
  // takes the next template item, a group or a token, as the spec, :287-288).
  // Undefined, the template reader stripped the spec's braces and read
  // `2pt/2pt` as columns (`p{t}` twice, calc then erroring on `t` as pdflatex
  // would on such a column: 2605.19920 9 errors).
  DefColumnType!(";{}", sub[(_dash_gap)] {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_between_column(dashed_rule_between_columns());
    });
  });
  // arydshln defines the first/last dashed rules only when array was loaded
  // first (arydshln.sty:66-68, :481-491); otherwise they stay undefined, and
  // the author's own command is the one reported.
  if lookup_meaning(&T_CS!("\\firsthline")).is_some() {
    DefMacro!("\\firsthdashline[]", "\\firsthline");
    DefMacro!("\\lasthdashline[]", "\\lasthline");
  }
  DefRegister!("\\dashlinedash" => Dimension!("4pt"));
  DefRegister!("\\dashlinegap" => Dimension!("4pt"));
  Let!("\\hdashlinewidth", "\\dashlinedash");
  Let!("\\hdashlinegap", "\\dashlinegap");
  def_macro_noop("\\ADLactivate")?;
  // `\def\ADLdrawingmode#1` (arydshln.sty:677) reads its mode number (K13 stage-2 finding).
  def_macro_noop("\\ADLdrawingmode{}")?;
  def_macro_noop("\\ADLinactivate")?;
  def_macro_noop("\\ADLnoshorthanded")?;
  def_macro_noop("\\ADLnullwide")?;
  def_macro_noop("\\ADLnullwidehline")?;
  def_macro_noop("\\ADLsomewide")?;
  def_macro_noop("\\ADLsomewidehline")?;
  // `\arrayrulecolor[model]{color}` and `\doublerulesepcolor[model]{color}` are
  // colortbl's commands (colortbl.sty.ltxml L85/L88: `[]{}`). The ar5iv arydshln
  // binding declares them 0-arg (`DefMacro('\arrayrulecolor', Tokens())`) — which
  // is order-fragile: it only behaves when colortbl loads AFTER arydshln (so
  // colortbl's `[]{}` overrides). When colortbl loads FIRST (e.g. pulled in
  // ahead of arydshln via the package dep graph — witness 1909.02323, mnras +
  // arydshln + colortbl), arydshln's 0-arg form is the final meaning and
  // `\arrayrulecolor{gray}\hline` leaves `{gray}` behind as a cell, so the
  // following `\hline` (= `\noalign{…}`) fires "\noalign cannot be used here".
  // Declare them with colortbl's `[]{}` arity so the color argument is consumed
  // regardless of load order — matching Perl's *effective* (colortbl-wins) result.
  DefMacro!("\\arrayrulecolor[]{}", None);
  def_macro_noop("\\dashgapcolor{}")?;
  DefMacro!("\\doublerulesepcolor[]{}", None);
  // NOTE: do NOT noop `\endlongtable`. The ar5iv Perl binding
  // (arydshln.sty.ltxml L45) does `DefMacro('\endlongtable', Tokens())`, but
  // that diverges from the REAL arydshln.sty, which SAVES and RESTORES
  // longtable's original `\endlongtable` (`\let\endlongtable\adl@org@endlongtable`,
  // arydshln.sty L796) rather than neutralizing it. Our longtable binding
  // relies on `\endlongtable` = `\lx@end@alignment\@end@tabular` to close the
  // alignment's boxing group; noop'ing it leaks that `{`-group so the
  // environment's `\endgroup` mismatches → mode cascade → `pop last locked
  // stack frame` FATAL (1510.04473: any `arydshln` + `longtable` with `p{}`
  // columns). Perl recovers from the same mismatch with 9 errors; our engine
  // aborts. Keeping longtable's `\endlongtable` functional matches the real
  // package and produces clean output (0 errors).
  def_macro_noop("\\nodashgapcolor")?;
  def_macro_noop("\\xleaders")?;
});

/// The between-column material of arydshln's dashed column rules (`:` and
/// `;{..}`): a `\vrule` classed `ltx_border_r_dashed`, which the stylesheet
/// draws dashed (ar5iv-bindings arydshln.sty.ltxml:21-24).
fn dashed_rule_between_columns() -> Vec<Token> {
  vec![
    T_CS!("\\vrule"),
    T_CS!("\\@ADDCLASS"),
    T_BEGIN!(),
    T_OTHER!("ltx_border_r_dashed"),
    T_END!(),
    T_CS!("\\relax"),
  ]
}
