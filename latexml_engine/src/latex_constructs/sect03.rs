//! `latex_constructs` section 3: C.3 Sentences and Paragraphs
//!
//! Split verbatim from the single `LoadDefinitions!` body (2026-09-03); the
//! definitions run in the original order from `super::load_definitions`.

use super::*;

#[rustfmt::skip]
pub(crate) fn load() -> Result<()> {
  // ======================================================================
  // C.3 Sentences and Paragraphs
  // ======================================================================

  //======================================================================
  // C.3.1 Making Sentences
  //======================================================================
  // quotes;  should these be handled in DOM/construction?
  // dashes:  We'll need some sort of Ligature analog, or something like
  // Omega's OTP, to combine sequences of "-" into endash, emdash,
  // Perhaps it also applies more semantically?
  // Such as interpreting certain sequences as section headings,
  // or math constructs.

  // Spacing; in TeX.pool.ltxml

  // Special Characters; in TeX.pool.ltxml

  // Logos
  // \TeX is in TeX.pool.ltxml
  DefMacro!("\\LaTeX", "LaTeX");
  DefMacro!("\\LaTeXe", "LaTeX2e");
  // Perl: enterHorizontal => 1
  DefConstructor!("\\LaTeX","<ltx:text class='ltx_LaTeX_logo' cssstyle='letter-spacing:-0.2em; margin-right:0.1em'
  >L<ltx:text cssstyle='font-variant:small-caps;' yoffset='0.4ex'
  >a</ltx:text
  >T<ltx:text cssstyle='font-variant:small-caps;font-size:120%' yoffset='-0.2ex'
  >e</ltx:text
  >X</ltx:text>",
  enter_horizontal => true,
  sizer => { Ok((Dimension!("2.6em"), Dimension!("1.6ex"), Dimension!("0.5ex"))) });

  // Perl: enterHorizontal => 1
  DefConstructor!("\\LaTeXe","<ltx:text class='ltx_LaTeX_logo' cssstyle='letter-spacing:-0.2em; margin-right:0.1em'
  >L<ltx:text cssstyle='font-variant:small-caps;' yoffset='0.4ex'
  >a</ltx:text
  >T<ltx:text cssstyle='font-variant:small-caps;font-size:120%' yoffset='-0.2ex'
  >e</ltx:text
  >X\u{2002}2<ltx:text cssstyle='font-style:italic' yoffset='-0.3ex'
  >\u{03B5}</ltx:text></ltx:text>",
  enter_horizontal => true,
  sizer => { Ok((Dimension!("3.7em"), Dimension!("1.6ex"), Dimension!("0.5ex"))) });

  // \fmtname / \fmtversion are intentionally NOT (re)defined here. Perl defines
  // them once, in latex_base.pool (↔ our latex_base.rs); this constructs-phase
  // copy was a Rust-only duplicate. Crucially, `constructs` runs AFTER the dump
  // apply (LoadFormat: bootstrap → dump → constructs), so re-hardcoding here
  // CLOBBERED the real per-TL-year kernel value the dump already carries
  // (e.g. \fmtversion 2025-11-01 on TL2025), pinning every \@ifl@t@r\fmtversion
  // check to the stale 2018/12/01 (issue #739). The dump is authoritative; the
  // no-dump/base path falls back to latex_base.rs's Perl-faithful value.

  DefMacro!("\\today", { ExplodeText!(Today!()) });

  // Use fonts (w/ special flag) to propogate emphasis as a font change,
  // but preserve it's "emph"-ness.
  // Perl latex_constructs.pool.ltxml L401-408: mode => 'restricted_horizontal',
  //   enterHorizontal => 1, font => { emph => 1 }, alias => '\emph', beforeDigest => {...}.
  // Perl emits `<ltx:text>` inside math context, `<ltx:emph>` outside —
  // dispatch via `findnodes('ancestor::ltx:Math')` in the body sub.
  // Rust template `?#isMath(...)(...)` is the equivalent gate (same
  // pattern used by `\newline`, `\thinspace`, etc.) — ported 2026-05-01.
  // The earlier note here claimed a Rust-only `$$`-in-`\emph{}` math
  // leak; verified 2026-05-03 it is SHARED-FAILURE with Perl, not a
  // Rust-only divergence. Both engines treat `$$` inside
  // `restricted_horizontal` as two inline-math toggles (per the
  // `BOUND_MODE =~ /vertical$/` gate in `\lx@dollar@default`). See
  // SYNC_STATUS.md Gate 2.A. mode "text" here is fine; flipping to
  // "restricted_horizontal" is a stylistic difference but does not
  // change the parity-relevant behavior.
  DefConstructor!("\\emph{}",
    "?#isMath(<ltx:text _force_font='1'>#1)(<ltx:emph _force_font='1'>#1)",
    // NB: no `mode => "text"` — Perl's \emph (latex_constructs.pool.ltxml:411) uses
    // only `enterHorizontal => 1, bounded => 1`. Adding `mode => "text"` digested
    // the argument in restricted_horizontal mode, so BOUND_MODE was not "vertical"
    // and the `$$` display-math probe in \lx@dollar@default was skipped — `$$…$$`
    // inside \emph{…} (common in theorem bodies) degraded to two empty inline `$`
    // and its sub/superscripts errored "Script _/^ can only appear in math mode"
    // (witness 2203.05327: 34 spurious errors; Perl 0).
    bounded        => true,
    enter_horizontal => true,
    font=> { emph => true },
    alias => "\\emph",
    before_digest => {
      if Expand!(T_CS!("\\f@shape")).eq_text("it") {
        DefMacro!(T_CS!("\\f@shape"), None, Tokens!(T_LETTER!("n")));
      } else {
        DefMacro!(T_CS!("\\f@shape"), None, Tokens!(T_LETTER!("i"),T_LETTER!("t")));
      }
    },
    after_construct => sub[doc,_args] {
      doc.maybe_close_element("ltx:emph")?; }
  );

  //======================================================================
  // C.3.2 Making Paragraphs
  //======================================================================
  // \noindent, \indent, \par in TeX.pool.ltxml

  Let!("\\@@par", "\\par");
  DefMacro!("\\@par", r"\let\par\@@par\par");
  DefMacro!("\\@restorepar", r"\def\par{\@par}");

  // Style parameters
  // \parindent, \baselineskip, \parskip alreadin in TeX.pool.ltxml

  def_primitive_noop("\\linespread{}")?;

  // NOTE: do NOT define `\geometry`/`\newgeometry` at the kernel level.
  // Perl only defines them when geometry.sty loads (geometry.sty.ltxml), so
  // `\ifcsname geometry\endcsname` is FALSE until then — a guard documents
  // legitimately use to detect whether the geometry package is loaded.
  // Defining `\geometry` unconditionally here made that guard always true:
  // witness 2005.03740, whose definitions.tex does
  // `\ifcsname geometry\endcsname \ingfxfiletrue \else \ingfxfilefalse
  // \usepackage{geometry}\fi` and then defines all its theorem environments
  // only in the \else branch — the false-positive guard skipped every
  // `\newtheorem` → cascade of "environment {theorem} is not defined" (Perl 0).
  // Classes/packages that genuinely page-set with geometry pull it in via
  // `\RequirePackage{geometry}` (geometry_sty.rs provides the no-op `\geometry`),
  // including the WileyMSP-template binding (the prior witness 2306.02129).

  // ?
  def_macro_noop("\\@noligs")?;
  DefConditional!("\\if@endpe");
  def_macro_noop("\\@doendpe")?;
  DefMacro!("\\@bsphack", "\\relax"); // what else?
  DefMacro!("\\@esphack", "\\relax");
  DefMacro!("\\@Esphack", "\\relax");

  //======================================================================
  // C.3.3 Footnotes
  //======================================================================

  // Footnote counters + \thefootnote / \thempfn / \thempfootnote +
  // \footnotesep register all defined in `latex_constructs_rust_only.rs`
  // section 8 (Perl `latex_base.pool.ltxml` L268-273; dump-path coverage
  // there is the authoritative copy).
  DefMacro!("\\footnotetyperefname", "footnote");

  def_macro_noop("\\ext@footnote")?;
  DefConstructor!("\\lx@note[]{}[]{}",
  "^<ltx:note role='#role' mark='#mark' xml:id='#id' inlist='#list'>#tags#4</ltx:note>",
  // Perl #2798: footnotes are inline blocks — internal_vertical, no leaveHorizontal.
  mode         => "inline_internal_vertical",
  before_digest => {
    neutralize_font(); },
  properties   => sub [args] {
    let arg1 = args[0].as_ref();
    let arg2 = args[1].as_ref();
    let arg3 = args[2].as_ref().map(Cow::Borrowed);
    let note_type = strip_trailing_cs(&arg2.as_ref().map(ToString::to_string).unwrap_or_default());
    let mut props = make_note_tags(&note_type, arg1, arg3)?;
    props.insert("list", digest_text(Tokens!(T_CS!(s!("\\ext@{note_type}"))))?.into());
    props.insert("role", note_type.into());
    Ok(props)
  },
  reversion => "");

  DefConstructor!("\\lx@notemark[]{}[]",
  "^<ltx:note role='#role' mark='#mark' xml:id='#id' inlist='#list'>#tags</ltx:note>",
  mode       => "text", enter_horizontal => true,
  properties => sub[args] {
    let arg1 = args[0].as_ref();
    let arg2 = args[1].as_ref();
    let arg3 = args[2].as_ref().map(Cow::Borrowed);
    let note_type = strip_trailing_cs(&arg2.as_ref().map(ToString::to_string).unwrap_or_default());
    let mut props = make_note_tags(&note_type, arg1, arg3)?;
    props.insert("role", s!("{note_type}mark").into());
    props.insert("list", digest_text(Tokens!(T_CS!(s!("\\ext@{note_type}"))))?.into());
    Ok(props)
  },
  reversion => "");

  // `OptionalSemiverbatim` on the first `[]` (the `xml:id`) so a paper
  // writing `\fntext[footnote_label2]{…}` (literal `_` in the label —
  // technically invalid TeX) doesn't blow up with `_ Script outside
  // math` when the digester sees the SUB-catcode `_`. Semiverbatim
  // reads `_` (and `^`, `~`, `&`, `$`, `#`, `'`) as OTHER, so the id
  // is a literal text token list. Paper-quality fix that surpasses
  // Perl LaTeXML (which still reads with default catcodes and errors
  // identically). Witness: 2604.00193 `\fntext[footnote_label2]` —
  // Rust before: 1 error; after: 0 errors. Same pattern surfaces on
  // most of the 79-paper math-mode-first cluster (SHARED-FAILURE
  // classification in SYNC_STATUS — this is the surpass-Perl path).
  DefConstructor!("\\lx@notetext OptionalSemiverbatim {} [] {}",
  "^<ltx:note role='#role' mark='#mark' xml:id='#id'>#4</ltx:note>",
  // Perl #2798: footnote text is an inline block — internal_vertical, no leaveHorizontal.
  mode       => "inline_internal_vertical",
  properties => sub [args] {
    let arg1 = args[0].as_ref();
    let arg2 = args[1].as_ref();
    let arg3 = args[2].as_ref();
    let note_type = strip_trailing_cs(&arg2.as_ref().map(ToString::to_string).unwrap_or_default());
    let arg3_ready = if let Some(v) = arg3 { Cow::Borrowed(v) } else {
      Cow::Owned(
        digest(T_CS!(s!("\\the{note_type}")))?
      )
    };
    let mut props = make_note_tags(&note_type, arg1, Some(arg3_ready))?;
    props.insert("role", s!("{note_type}text").into());
    Ok(props)
  },
  reversion => "");

  DefConstructor!("\\lx@note@live OptionalSemiverbatim {} []",
  "^<ltx:note role='#role' mark='#mark' xml:id='#id' inlist='#list'>#tags#body</ltx:note>",
  before_digest => {
    neutralize_font();
    begin_mode("inline_internal_vertical")?;
  },
  properties   => sub [args] {
    let arg1 = args[0].as_ref();
    let arg2 = args[1].as_ref();
    let arg3 = args[2].as_ref().map(Cow::Borrowed);
    let note_type = strip_trailing_cs(&arg2.as_ref().map(ToString::to_string).unwrap_or_default());
    let mut props = make_note_tags(&note_type, arg1, arg3)?;
    props.insert("list", digest_text(Tokens!(T_CS!(s!("\\ext@{note_type}"))))?.into());
    props.insert("role", note_type.into());
    Ok(props)
  },
  capture_body => true,
  reversion => "");

  DefConstructor!("\\lx@notetext@live OptionalSemiverbatim {} []",
  "^<ltx:note role='#role' mark='#mark' xml:id='#id'>#body</ltx:note>",
  before_digest => {
    neutralize_font();
    begin_mode("inline_internal_vertical")?;
  },
  properties => sub [args] {
    let arg1 = args[0].as_ref();
    let arg2 = args[1].as_ref();
    let arg3 = args[2].as_ref();
    let note_type = strip_trailing_cs(&arg2.as_ref().map(ToString::to_string).unwrap_or_default());
    let arg3_ready = if let Some(v) = arg3 { Cow::Borrowed(v) } else {
      Cow::Owned(
        digest(T_CS!(s!("\\the{note_type}")))?
      )
    };
    let mut props = make_note_tags(&note_type, arg1, Some(arg3_ready))?;
    props.insert("role", s!("{note_type}text").into());
    Ok(props)
  },
  capture_body => true,
  reversion => "");

  DefConstructor!(T_CS!("\\lx@note@live@end"), None, None,
    before_digest => {
      end_mode("inline_internal_vertical")?;
    });

  DefMacro!("\\lx@note@standard", "\\lx@note{footnote}");
  DefMacro!("\\lx@notetext@standard", "\\lx@notetext{footnote}");
  DefMacro!("\\lx@current@footnote", "\\lx@note@standard");
  DefMacro!("\\lx@current@footnotetext", "\\lx@notetext@standard");

  // OXIDIZED_DESIGN #192: fancyvrb/fancybox expose the public command; the
  // kernel owns only the locked activation helper so the package surface does
  // not leak into documents that loaded neither package.
  Let!("\\lx@temp", "\\relax");

  DefMacro!("\\lx@VerbatimFootnotes",
    "\\let\\lx@current@footnote\\lx@vfootnote\\let\\lx@current@footnotetext\\lx@vfootnotetext",
    locked => true);

  DefMacro!("\\lx@vfootnote",
    "\\@ifnextchar[{\\lx@vfootnote@opt}{\\lx@vfootnote@noopt}");
  DefMacro!("\\lx@vfootnote@opt[]",
    "\\lx@note@live{footnote}[#1]\\afterassignment\\lx@vfootnote@start\\let\\lx@temp");
  DefMacro!("\\lx@vfootnote@noopt",
    "\\lx@note@live{footnote}\\afterassignment\\lx@vfootnote@start\\let\\lx@temp");
  DefMacro!("\\lx@vfootnote@start",
    "\\bgroup\\aftergroup\\lx@note@live@end");

  DefMacro!("\\lx@vfootnotetext",
    "\\@ifnextchar[{\\lx@vfootnotetext@opt}{\\lx@vfootnotetext@noopt}");
  DefMacro!("\\lx@vfootnotetext@opt[]",
    "\\lx@notetext@live{footnote}[#1]\\afterassignment\\lx@vfootnote@start\\let\\lx@temp");
  DefMacro!("\\lx@vfootnotetext@noopt",
    "\\lx@notetext@live{footnote}\\afterassignment\\lx@vfootnote@start\\let\\lx@temp");

  DefMacro!("\\footnote", "\\lx@current@footnote", locked => true);
  DefMacro!("\\footnotemark",  "\\lx@notemark{footnote}", locked => true);
  DefMacro!("\\footnotetext", "\\lx@current@footnotetext", locked => true);
  DefMacro!("\\@footnotetext", "\\lx@current@footnotetext", locked => true);
  // we don't implement the internals directly, so lock them to the latexml variant
  Let!("\\@thefnmark", "\\lx@notemark{footnote}");

  // \@makefntext: article.cls L207/609 wraps the footnote body content
  // (mark + text) for emission inside the footnote area. Packages like
  // babel hyphenrules and class-conditional code reference it before
  // the class loads its definition. Provide a content-preserving stub
  // so the body (#1) is emitted as plain text rather than triggering
  // an undefined-CS error. Witness: 2503.15258 (elsarticle via babel)
  // and 2503.16849 (ieeeconf via babel).
  def_macro_identity("\\@makefntext{}")?;
  DefMacro!("\\@makefnmark", "\\@thefnmark");

  Tag!("ltx:emph", auto_close => true);
  Tag!("ltx:note", after_close => sub[doc, node] { relocate_footnote(doc, node)?; });

  // Style parameters
  // \footnotesep register lives in `latex_constructs_rust_only.rs` section 8.
  def_primitive_noop("\\footnoterule")?;

  Ok(())
}
