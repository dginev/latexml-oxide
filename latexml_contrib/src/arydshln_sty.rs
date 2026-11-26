//! arydshln.sty: dashed horizontal and vertical rules in `array` and `tabular`. After upstream LaTeXML PR 2889 by
//! Christopher Smolen (arydshln.sty.ltxml), with the rules drawn dashed: each side a dashed rule borders is a
//! `border` letter, as for a solid one, and the cell gets the class `ltx_border_<side>_dashed` (OD #1025).
use latexml_engine::tex_tables::{alignment_to_rule, cline_columns};
use latexml_package::{package::colortbl_sty::rule_paint, prelude::*};

LoadDefinitions!({
  //======================================================================
  // Registers
  DefRegister!("\\dashlinedash" => Dimension!("4pt"));
  DefRegister!("\\dashlinegap" => Dimension!("4pt"));
  Let!("\\hdashlinewidth", "\\dashlinedash");
  Let!("\\hdashlinegap", "\\dashlinegap");

  //======================================================================
  // Horizontal dashed rules. Each reads an optional `[<dash>/<gap>]` (arydshln.sty:432-438 `\adl@hdashline` →
  // `\@ifnextchar[`, :459-462 `\cdashline#1` likewise), dropped: the stylesheet draws the dashes. As `\let`s of
  // `\hline`/`\cline` (KNOWN_PERL_ERRORS #291) the `[2pt/2pt]` was left as text in the next row's first cell
  // (2605.19920). The rule takes colortbl's `\arrayrulecolor` as `\hline` does (arydshln.sty:1110-1112 `\CT@arc@`).
  DefMacro!(
    "\\lx@adl@arc",
    r"\ifx\lx@CT@arc@draw\@undefined\else\lx@CT@arc@draw\fi"
  );
  DefMacro!("\\hdashline[]", r"\noalign{\lx@adl@arc\lx@adl@hdashline}");
  DefConstructor!("\\lx@adl@hdashline", "",
    after_digest => sub[_whatsit] {
      if let Some(alignment) = alignment_to_rule() {
        alignment.alignment_cell().unwrap().borrow_mut().add_dashed_line("t", Vec::new());
      }
    },
    properties => { Ok(stored_map!("isHorizontalRule" => true)) },
    sizer => 0, alias => "\\hdashline");
  // `\cdashline{a-b}[dash/gap]`, its columns read as `\cline`'s.
  DefMacro!(
    "\\cdashline{}[]",
    r"\noalign{\lx@adl@arc\lx@adl@cdashline{#1}}"
  );
  DefConstructor!("\\lx@adl@cdashline{}", "",
    after_digest => sub[whatsit] {
      let cols = whatsit.get_arg(1).map(ToString::to_string).unwrap_or_default();
      if let Some(alignment) = alignment_to_rule() {
        alignment.alignment_cell().unwrap().borrow_mut().add_dashed_line("t", cline_columns(&cols));
      }
    },
    sizer => 0, alias => "\\cdashline");
  // arydshln defines the first/last dashed rules only when array was loaded first (arydshln.sty:66-68, :481-491);
  // otherwise they stay undefined, and the author's own command is the one reported.
  if lookup_meaning(&T_CS!("\\firsthline")).is_some() {
    DefMacro!(
      "\\firsthdashline[]",
      r"\noalign{\lx@adl@arc\lx@adl@hdashline}"
    );
    DefMacro!(
      "\\lasthdashline[]",
      r"\noalign{\lx@adl@arc\lx@adl@hdashline}"
    );
  }

  //======================================================================
  // Vertical dashed rules: the `:` and `;{<dash>/<gap>}` column types. Like `|`, each puts a rule between the
  // columns, which the cell-edge scan (tex_tables.rs `extract_alignment_column`) turns into an `l` or `r` border of
  // the cell it ends, dashed.
  DefConstructor!("\\lx@adl@vdashrule", "",
    properties => { Ok(stored_map!("isVerticalRule" => true, "border_dashed" => true)) },
    sizer => 0, reversion => "");
  alignment::declare_column_rule_token("\\lx@adl@vdashrule");
  DefColumnType!(":", {
    let rule = dashed_rule();
    with_building_template(|template| template.add_between_column(rule));
  });
  // `;{<dash>/<gap>}` is `:` with its own dash pattern (arydshln.sty:198/236 map `;` to `\adl@argarraydashrule`,
  // :270-273, whose `\adl@classvfordash` takes the next template item, a group or a token, as the spec, :287-288).
  // Undefined, the template reader stripped the spec's braces and read `2pt/2pt` as columns (`p{t}` twice, calc then
  // erroring on `t` as pdflatex would on such a column: 2605.19920 9 errors).
  DefColumnType!(";{}", sub[(_dash_gap)] {
    let rule = dashed_rule();
    with_building_template(|template| template.add_between_column(rule));
  });

  //======================================================================
  // arydshln.sty:697-729 the shorthand environments `{Array}`, `{Tabular}`, `{Tabular*}` (and `{Longtable}` when
  // longtable came first) are its tables with the dashed lines active, defined unless one of the names is taken
  // (then a warning, :716-729). Here every table draws the dashed lines, so each is the plain environment, `\let`
  // as at :702-710 (witnesses 2502.17812 `{Tabular}`, 2306.04970 `{Array}`: 5 and 1 errors before, 0 after).
  RawTeX!(
    r"\begingroup\def\@notdefinable{\gdef\adl@notdefinable{}}%
\@ifdefinable\Array\relax\@ifdefinable\Tabular\relax\expandafter\@ifdefinable\csname Tabular*\endcsname\relax
\ifx\longtable\undefined\else\@ifdefinable\Longtable\relax\fi\endgroup
\ifx\adl@notdefinable\undefined
  \let\Array\array\let\endArray\endarray\let\Tabular\tabular\let\endTabular\endtabular
  \expandafter\let\csname Tabular*\expandafter\endcsname\csname tabular*\endcsname
  \expandafter\let\csname endTabular*\expandafter\endcsname\csname endtabular*\endcsname
  \ifx\longtable\undefined\else\let\Longtable\longtable\def\endLongtable{\endlongtable}\fi
\else
  \PackageWarning{arydshln}{Array/Tabular/Longtable are not defined because\MessageBreak one of them has been defined}%
\fi"
  );
  // arydshln.sty:731-740 `\ADLnoshorthanded` undefines the shorthands.
  RawTeX!(
    r"\def\ADLnoshorthanded{\let\Array\relax\let\Tabular\relax\expandafter\let\csname Tabular*\endcsname\relax
\let\endArray\relax\let\endTabular\relax\expandafter\let\csname endTabular*\endcsname\relax
\ifx\longtable\undefined\else\let\Longtable\relax\let\endLongtable\relax\fi}"
  );
  // NOTE: do NOT noop `\endlongtable` (ar5iv's arydshln.sty.ltxml did): the real arydshln.sty keeps longtable's
  // (`\let\endlongtable\adl@org@endlongtable`, :796), which closes the alignment's boxing group; a no-op leaked that
  // group, a mode cascade ending in the `pop last locked stack frame` Fatal (1510.04473: arydshln + longtable with
  // `p{}` columns; Perl recovered with 9 errors).

  //======================================================================
  // Configuration of the TeX drawing: no effect here. `\def\ADLdrawingmode#1` (arydshln.sty:677) reads its mode.
  def_macro_noop("\\ADLactivate")?;
  def_macro_noop("\\ADLinactivate")?;
  def_macro_noop("\\ADLnullwide")?;
  def_macro_noop("\\ADLsomewide")?;
  def_macro_noop("\\ADLnullwidehline")?;
  def_macro_noop("\\ADLsomewidehline")?;
  def_macro_noop("\\ADLdrawingmode{}")?;
  // arydshln.sty:1053-1125: only when colortbl came first, `\arrayrulecolor` and `\doublerulesepcolor` are colortbl's,
  // redefined alike (`\adl@defcolor\CT@arc@`): colortbl's binding keeps them (1909.02323, mnras + colortbl + arydshln).
  // So are the gap colour setters (:1121, :1125), which have no CSS form: `\dashgapcolor` reads `[<model>]{<colour>}`.
  if package_loaded("colortbl") {
    DefMacro!("\\dashgapcolor[]{}", "");
    def_macro_noop("\\nodashgapcolor")?;
  }
});

/// The template's dashed column rule, painted in colortbl's rule colour as colortbl's `|` is (arydshln.sty:282-284
/// `\adl@vlineL{\CT@arc@}`).
fn dashed_rule() -> Vec<Token> {
  let mut rule = vec![T_CS!("\\lx@adl@vdashrule")];
  if package_loaded("colortbl") {
    rule.extend(rule_paint());
  }
  rule
}
