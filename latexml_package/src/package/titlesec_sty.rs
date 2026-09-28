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

  // `\titleformat*{\cmd}{format}` replaces only the format (titlesec.sty:672-683), keeping the
  // label and shape. Perl (titlesec.sty.ltxml:30-34) redefines the whole composer as
  // `<format> #1`, dropping the number (witness 2605.21802, KPE #328). A `\titleformat` already
  // given keeps its composer and shape class; otherwise titlesec's own default for the command
  // applies (titlesec.sty:1541-1563 `\ttl@@extract`, run for `\section`…`\subparagraph` at
  // :1631-1635): `\titleformat\cmd[runin or hang]{format}{\@seccntformat{cmd}}{0pt}` — the
  // format then covers label and title, as titlesec sets them (:740-795). Other commands (a
  // `\chapter`) take the format as their title font.
  DefMacro!("\\lx@titleformat@star {}{}", sub[(cmd, format)] {
    let cs_str = cmd.to_string();
    let sec = cs_str.strip_prefix('\\').unwrap_or(&cs_str).to_string();
    let extracted = matches!(
      sec.as_str(),
      "section" | "subsection" | "subsubsection" | "paragraph" | "subparagraph"
    );
    let mut tokens: Vec<Token> = Vec::new();
    if extracted && !lookup_bool(&s!("titlesec_formatted@{sec}")) {
      // titlesec picks runin by the sign of the class's `\@startsection` after-skip (:1552-1553):
      // negative for the standard classes' `\paragraph`/`\subparagraph`.
      let shape = if sec.ends_with("paragraph") { "runin" } else { "hang" };
      tokens.push(T_CS!("\\lx@titleformat"));
      tokens.push(T_BEGIN!());
      tokens.extend(cmd.unlist());
      tokens.push(T_END!());
      tokens.push(T_OTHER!("["));
      tokens.extend(Explode!(shape));
      tokens.push(T_OTHER!("]"));
      tokens.push(T_BEGIN!());
      tokens.extend(format.unlist());
      tokens.push(T_END!());
      tokens.extend([T_BEGIN!(), T_CS!("\\@seccntformat"), T_BEGIN!()]);
      tokens.extend(Explode!(sec.as_str()));
      tokens.extend([T_END!(), T_END!(), T_BEGIN!()]);
      tokens.extend(Explode!("0pt"));
      tokens.extend([T_END!(), T_BEGIN!(), T_END!(), T_OTHER!("["), T_OTHER!("]")]);
    } else {
      tokens.push(T_CS!("\\lx@titleformat@font"));
      tokens.push(T_BEGIN!());
      tokens.extend(Explode!(sec.as_str()));
      tokens.push(T_END!());
      tokens.push(T_BEGIN!());
      tokens.extend(format.unlist());
      tokens.push(T_END!());
    }
    Ok(Tokens::new(tokens))
  });
  // The format alone, with the shape class a `\titleformat` recorded.
  DefPrimitive!("\\lx@titleformat@font {}{}", sub[(sec, format)] {
    let sec = sec.to_string();
    let mut font_body: Vec<Token> = Vec::new();
    let cls = lookup_string(&s!("titlesec_shape_class@{sec}"));
    if !cls.is_empty() {
      font_body.push(T_CS!("\\lx@add@cssclass"));
      font_body.push(T_OTHER!(&cls));
    }
    font_body.extend(format.unlist());
    def_macro(T_CS!(&s!("\\format@title@font@{sec}")), None, Tokens::new(font_body), None)?;
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
    // The shape's class goes first: a format may end in a macro that takes the title's next
    // token (`{\bfseries\MakeUppercase}`), which would otherwise take `\lx@add@cssclass`.
    let mut font_body: Vec<Token> = Vec::new();
    if let Some(cls) = class {
      font_body.push(T_CS!("\\lx@add@cssclass"));
      font_body.push(T_OTHER!(cls));
    }
    font_body.extend(format.unlist());
    // A later `\titleformat*` keeps this shape (and a shape without a class clears an earlier one).
    assign_value(&s!("titlesec_shape_class@{sec}"), class.unwrap_or("").to_string(), None);
    assign_value(&s!("titlesec_formatted@{sec}"), true, None);
    def_macro(T_CS!(&font_target), None, Tokens::new(font_body), None)?;

    // \format@title@<sec>   (1 arg body)
    let body_target = s!("\\format@title@{sec}");
    let mut body: Vec<Token> = Vec::new();
    // titlesec.sty:420 (`\ttl@straight@i`): `\gdef\thetitle{\csname the#1\endcsname}`
    // when a title is typeset, so a label like mla.cls:196 `\thetitle.` works.
    body.extend(mouth::tokenize_internal(TeXString::assembled(s!(
      "\\gdef\\thetitle{{\\csname the{sec}\\endcsname}}"
    ))).unlist());
    // titlesec runs `<format>` once, before the label, setting label and title inside it
    // (titlesec.sty:740-795 `\ttlh@display`/`\ttlh@hang`/`\ttlh@runin`, :797-818 `\ttlhx@block`);
    // the kernel's `\lx@format@title@@` applies `\format@title@font@<sec>` to the title again
    // (base_utilities.rs, Perl Base_Utility.pool.ltxml:1099-1101) — visible material printed twice
    // ("XX1 XXIntro"), an alignment in the format dropped from numbered titles (KPE #327). Run it
    // from a copy and empty the font macro for the rest of this title's group; the copy keeps a
    // format ending in an argument-taking macro (`{\bfseries\MakeUppercase}`) off the `\let`.
    body.extend([
      T_CS!("\\let"), T_CS!("\\lx@titlesec@format"), T_CS!(&font_target),
      T_CS!("\\let"), T_CS!(&font_target), T_CS!("\\@empty"),
      T_CS!("\\lx@titlesec@format"),
    ]);
    // The `hang` (default) and `runin` shapes hand the format label, separator and title as one
    // group (titlesec.sty:767 `#1{…}`, :788), so a format ending in `\MakeUppercase` takes them
    // all; `display` and `block` set them after it (:748, :806).
    let grouped = matches!(shape_str.as_str(), "" | "hang" | "runin");
    if grouped {
      body.push(T_BEGIN!());
    }
    body.extend(label.unlist());
    body.push(T_CS!("\\hspace"));
    body.push(T_BEGIN!());
    body.extend(sep.unlist());
    body.push(T_END!());
    body.push(T_PARAM!());
    body.push(T_OTHER!("1"));
    if grouped {
      body.push(T_END!());
    }
    def_macro(T_CS!(&body_target), convert_latex_args(1, None)?,
      Tokens::new(body), None)?;
  });

  DefMacro!("\\chaptertitlename",                        "\\chaptername");
  def_macro_noop("\\titlespacing OptionalMatch:* {}{}{}{}[]")?;
  // titlesec stores each title's spacing in `\ttls@<section>` =
  // `{left}{right}{before}{after}{afterindent}` (titlesec.sty:640-658) and
  // fills it at load for the five standard sectioning commands
  // (`\ttl@extract`, :1579-1628). Ours are not built on `\@startsection`, so
  // titlesec's own fallback for such a command applies (:1587-1590
  // `\titlespacing*#1{\z@}{*3}{*2}`). The spacing is not rendered, but ctex
  // reads the record at the end of titlesec's load: ctex-heading-article.def:
  // 490-528 `\__ctex_titlesec_spacing:nnnnnn` takes its five groups plus the
  // name, and on an undefined record ran away across the headings map into a
  // `\csname CTEX@…` (mynsfc, qyxf-book, bjfuthesis; TeX Live class census
  // 2026-09-24; Perl's binding lacks the record too). ctex-heading-book.def:669
  // also resets `\ttl@chapterout` (titlesec.sty:402).
  for name in ["section", "subsection", "subsubsection", "paragraph", "subparagraph"] {
    def_macro(T_CS!(s!("\\ttls@{name}")), None,
      TokenizeInternal!(r"{\z@}{\z@}{*3}{*2}{\z@}"), None)?;
  }
  DefMacro!("\\ttl@chapterout", "\\typeout{\\chaptertitlename\\space\\thechapter.}");

  DefMacro!("\\filright",  "\\raggedright");
  DefMacro!("\\filcenter", "\\centering");
  DefMacro!("\\filleft",   "\\raggedleft");
  def_macro_noop("\\fillast")?;
  DefMacro!("\\filinner",  "\\filleft");
  DefMacro!("\\filouter",  "\\filright");
  // titlesec.sty:1164-1165: an interword glue, the font's space, stretch and shrink; Perl's
  // `DefRegister('\wordsep', Dimension(0))` (titlesec.sty.ltxml:68) set a run-in title flush
  // against its number (KPE #317).
  DefMacro!("\\wordsep", "\\fontdimen\\tw@\\font \\@plus \\fontdimen\\thr@@\\font \\@minus \\fontdimen4\\font");
  // titlesec.sty:1039-1041 `\newdimen\titlewidth`, `\titlewidthlast`,
  // `\titlewidthfirst` — set by the `calcwidth` machinery and read in user
  // title formats (titlesec.tex:1780 `\addtolength{\titlewidth}{2pc}`).
  // Guard: `perfect_kernel_batch54::titlesec_title_width_registers_exist`.
  DefRegister!("\\titlewidth", Dimension(0));
  DefRegister!("\\titlewidthlast", Dimension(0));
  DefRegister!("\\titlewidthfirst", Dimension(0));

  // titlesec.sty:1088-1095 `\titleline*[align]{material}` (the star, then `[s]`); Perl's
  // `[]{}` (titlesec.sty.ltxml:70) took the star as the material and left `[c]` in the title
  // (the titlesec manual's own example, titlesec.tex:1779-1793; KPE #317). The material is a line
  // of its own in the title (titlesec.sty:1095-1109): the material, then a line break — none when
  // it sets nothing (rule-only `{\titlerule*…}`, the rules being layout). The inner group scopes
  // a `\small` in it (a constructor's argument is not a group); `[align]` is layout.
  DefMacro!("\\titleline OptionalMatch:* []{}", "\\lx@titleline{{#3}}");
  DefConstructor!("\\lx@titleline{}", "#1?#1(<ltx:break/>)()");
  DefMacro!("\\titlerule", "\\@ifstar{\\lx@titlerule@star}{\\lx@titlerule}");
  def_macro_noop("\\lx@titlerule@star []{}")?;
  def_macro_noop("\\lx@titlerule []")?;

  // titlesec.sty:1047 `\let\iftitlemeasuring\@secondoftwo`: a two-branch choice, the first only
  // while titlesec measures a title, which LaTeXML never does. Perl's `DefConditional`
  // (titlesec.sty.ltxml:75) skipped both branches and the title to a missing `\fi` (KPE #317).
  Let!("\\iftitlemeasuring", "\\@secondoftwo");
  def_macro_noop("\\assignpagestyle{}{}")?;
  def_macro_noop("\\sectionbreak")?;
  def_macro_noop("\\subsectionbreak")?;
  def_macro_noop("\\subsubsectionbreak")?;
  def_macro_noop("\\paragraphbreak")?;
  def_macro_noop("\\subparagraphbreak")?;

  // titlesec.sty:112-165 `\titleclass{\cmd}[level]{class}[parent]`: for a NEW
  // command the class layer `\edef`s it (:138-139) and records its level in
  // `\ttll@<name>` (:122, or parent's +1 through `\ttl@class@iv`). The
  // former no-op left regulatory.sty:116/121 `\titleclass{\article}[0]
  // {straight}` / `\titleclass{\para}{straight}[\article]` undefined
  // (regulatory example1/2 -en/-nl; Perl titlesec.sty.ltxml:83 shares the
  // no-op). A kernel sectioning command (locked) keeps its binding; a new
  // one becomes an `\@startsection` heading at the titlesec level, shifted
  // by one in a chapterless class (LaTeX's own level 0 is `\chapter`). Guard:
  // `perfect_kernel_batch54::titleclass_defines_a_new_heading_command`.
  DefPrimitive!("\\titleclass {} [] {} []", sub[(cmd, level, _class, parent)] {
    let Some(cmd) = cmd.unlist().into_iter().find(|t| t.get_catcode() == Catcode::CS) else {
      return Ok(vec![]);
    };
    let name = cmd.with_str(|s| s.trim_start_matches('\\').to_string());
    let ttll: i64 = if let Some(level) = level.as_ref() {
      level.to_string().trim().parse().unwrap_or(0)
    } else if let Some(parent) = parent.as_ref() {
      let pname = parent.to_string();
      let pname = pname.trim().trim_start_matches('\\');
      do_expand(Tokenize!(TeXString::assembled(s!("\\csname ttll@{pname}\\endcsname"))))
        .map(|t| t.to_string().trim().parse::<i64>().unwrap_or(0) + 1)
        .unwrap_or(1)
    } else {
      // an existing level's class change: nothing to (re)define here
      return Ok(vec![]);
    };
    def_macro(T_CS!(s!("\\ttll@{name}")), None, Tokens::new(ExplodeText!(s!("{ttll}"))), None)?;
    if lookup_meaning(&cmd).is_some() {
      return Ok(vec![]); // \section & co keep their (locked) bindings
    }
    let shift = if lookup_definition(&T_CS!("\\c@chapter"))?.is_none() { 1 } else { 0 };
    def_macro(
      cmd,
      None,
      mouth::tokenize_internal(TeXString::assembled(
        s!("\\@startsection{{{name}}}{{{}}}{{}}{{}}{{}}{{}}", ttll + shift))),
      None,
    )?;
  });

  // titlesec.sty L1178 + L1385-1422: the `pagestyles` option (also the
  // deprecated `psfloats`/`pagegrids` aliases) makes titlesec
  // `\input{titleps.sty}` at the end of its load. Mirror that with the
  // titleps binding. Witness: ufrgscca-abnt.sty L133
  // `\RequirePackage[pagestyles,clearempty]{titlesec}` (perfect-kernel).
  let titlesec_opts = do_expand(Tokenize!(r"\csname opt@titlesec.sty\endcsname"))
    .map(|t| t.to_string())
    .unwrap_or_default();
  if titlesec_opts.contains("pagestyles") || titlesec_opts.contains("pagegrids") {
    RequirePackage!("titleps");
  }
});
