use crate::prelude::*;

// Perl titlesec.sty.ltxml L35-40: titlesec "shape" option → CSS class.
// Rust inlines the same four-entry map at each lookup site so both
// primitives reference a single source of truth.
fn titlesec_shape_class(shape: &str) -> Option<&'static str> {
  match shape {
    "runin" => Some("ltx_runin"),
    "frame" => Some("ltx_framed ltx_framed_rectangle"),
    "rightmargin" => Some("ltx_align_right"),
    "leftmargin" => Some("ltx_align_left"),
    _ => None,
  }
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: titlesec.sty.ltxml — stubbed since no styling was implemented,
  // but the star + non-star forms actually DO dynamic-macro work in Perl.
  // This cycle brings those to parity.

  def_macro_noop("\\titlelabel{}")?;
  // \titleformat: star and normal forms
  DefMacro!("\\titleformat", "\\@ifstar{\\lx@titleformat@star}{\\lx@titleformat}");

  // Perl L30-34: \titleformat*{\cmd}{format} redefines
  // `\format@title@<cmd>` to `<format> <space> #1` with 1 parameter.
  // Users writing \titleformat*{\section}{\bfseries} get a working
  // override instead of a silent drop. Strip leading backslash from
  // the command name per Perl L32.
  // Perl kind is DefMacro with sub body that installs a macro via
  // DefMacroI. Rust DefPrimitive does the install at stomach time.
  // WISDOM #44: NOT universally equivalent — safe here because
  // `\lx@titleformat@star` is only invoked via `\titleformat*{\cmd}{format}`
  // at preamble/document time, never captured by `\edef`.
  // WISDOM #44 verified 2026-04-23: zero `\edef`/`\ifx`/`\expandafter`
  // uses of `\lx@titleformat@star` across LaTeXML/lib + ar5iv-bindings.
  DefPrimitive!("\\lx@titleformat@star {}{}", sub[(cmd, format)] {
    let cs_str = cmd.to_string();
    let sec = cs_str.strip_prefix('\\').unwrap_or(&cs_str);
    let target = s!("\\format@title@{sec}");
    let mut body: Vec<Token> = format.unlist();
    body.push(T_SPACE!());
    body.push(T_PARAM!());
    body.push(T_OTHER!("1"));
    def_macro(T_CS!(&target), convert_latex_args(1, None)?,
      Tokens::new(body), None)?;
  });

  // Perl L42-57: \titleformat{cmd}[shape]{format}{label}{sep}{before}[after]
  // Ignores before/after (Perl too). If shape maps to a CSS class, inject
  // `\@ADDCLASS{<class>}` after the format tokens. Then defines two
  // macros:
  //   \format@title@font@<sec>        := <format> [\@ADDCLASS <class>]
  //   \format@title@<sec>   ( 1 arg ) := \format@title@font@<sec> <label>
  //                                      \hspace{<sep>} #1
  DefPrimitive!("\\lx@titleformat {} [] {}{}{}{}[]",
    sub[(cmd, shape, format, label, sep, _before, _after)] {
    let cs_str = cmd.to_string();
    let sec = cs_str.strip_prefix('\\').unwrap_or(&cs_str);
    let shape_str = shape.as_ref().map(|s| s.to_string()).unwrap_or_default();
    let class = titlesec_shape_class(&shape_str);

    // \format@title@font@<sec>
    let font_target = s!("\\format@title@font@{sec}");
    let mut font_body: Vec<Token> = format.unlist();
    if let Some(cls) = class {
      font_body.push(T_CS!("\\@ADDCLASS"));
      font_body.push(T_OTHER!(cls));
    }
    def_macro(T_CS!(&font_target), None, Tokens::new(font_body), None)?;

    // \format@title@<sec>   (1 arg body)
    let body_target = s!("\\format@title@{sec}");
    let mut body: Vec<Token> = Vec::new();
    body.push(T_CS!(&font_target));
    body.extend(label.unlist());
    body.push(T_CS!("\\hspace"));
    body.push(T_BEGIN!());
    body.extend(sep.unlist());
    body.push(T_END!());
    body.push(T_PARAM!());
    body.push(T_OTHER!("1"));
    def_macro(T_CS!(&body_target), convert_latex_args(1, None)?,
      Tokens::new(body), None)?;
  });

  DefMacro!("\\chaptertitlename",                        "\\chaptername");
  def_macro_noop("\\titlespacing OptionalMatch:* {}{}{}{}[]")?;

  DefMacro!("\\filright",  "\\raggedright");
  DefMacro!("\\filcenter", "\\centering");
  DefMacro!("\\filleft",   "\\raggedleft");
  def_macro_noop("\\fillast")?;
  DefMacro!("\\filinner",  "\\filleft");
  DefMacro!("\\filouter",  "\\filright");
  DefRegister!("\\wordsep", Dimension(0));

  def_macro_noop("\\titleline[]{}")?;
  DefMacro!("\\titlerule", "\\@ifstar{\\lx@titlerule@star}{\\lx@titlerule}");
  def_macro_noop("\\lx@titlerule@star []{}")?;
  def_macro_noop("\\lx@titlerule []")?;

  DefConditional!("\\iftitlemeasuring");
  def_macro_noop("\\assignpagestyle{}{}")?;
  def_macro_noop("\\sectionbreak")?;
  def_macro_noop("\\subsectionbreak")?;
  def_macro_noop("\\subsubsectionbreak")?;
  def_macro_noop("\\paragraphbreak")?;
  def_macro_noop("\\subparagraphbreak")?;

  def_macro_noop("\\titleclass{}[]{} []")?;
});
