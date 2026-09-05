use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: lineno.sty.ltxml — stub (line numbering not meaningful for XML)
  DefEnvironment!("{linenumbers*}[Number]",         "#body");
  DefEnvironment!("{runninglinenumbers*}[Number]",  "#body");
  DefEnvironment!("{pagewiselinenumbers*}[Number]", "#body");
  DefEnvironment!("{linenomath}",                   "#body");
  DefEnvironment!("{linenomath*}",                  "#body");
  // lineno.sty:2881 `bframe` — a framed block (frame presentational; ulineno).
  DefEnvironment!("{bframe}", "#body");
  DefRegister!("\\bframerule", Dimension(26214)); // \fboxrule = 0.4pt
  DefRegister!("\\bframesep",  Dimension(196608)); // \fboxsep = 3pt

  // Real lineno.sty also defines control sequences `\linenomath`,
  // `\linenomathWithnumbers`, `\linenomathNonumbers` (raw-load
  // sees these as macros). Other packages — eccv.sty, journal templates —
  // test them with `\ifx\linenomath\linenomathWithnumbers` to switch
  // between AMS-math styles. Without explicit defs here, all three resolve
  // to `\relax` and the `\ifx` test is TRUE — the then-branch fires
  // `\patchcmd\linenomathAMS{...}` which is undefined → cascade of
  // `\else` / `\fi` mismatch (27 of 44 wp4 \else-error papers use eccv).
  // Make them three *distinct* no-op macros so the `\ifx` test picks the
  // else-branch reliably, matching the no-linenumbers default.
  // Don't redefine `\linenomath` / `\endlinenomath` — those are the
  // env-begin/env-end macros set up by DefEnvironment above. We DO
  // define the two "style switch" macros that real lineno provides,
  // with distinct bodies so journal-template `\ifx\linenomath\linenomathWithnumbers`
  // tests reliably pick the no-linenumbers branch.
  DefMacro!("\\linenomathWithnumbers", "\\relax");
  DefMacro!("\\linenomathNonumbers",   "\\@empty");

  // \internallinenumbers (lineno.sty:2732) is a macro with optional * and [Number].
  // lineno.sty also defines `\let\endinternallinenumbers\endlinenumbers` and
  // `\@namedef{internallinenumbers*}{\internallinenumbers*}` so it can be used
  // BOTH as a macro (inside boxes/parboxes; ulineno.tex:902) and as an environment
  // (\begin{internallinenumbers}; iclr2025_conference.sty, aastex, fvextra).
  // A DefEnvironment here breaks macro calls by entering restricted_horizontal mode
  // and opening an unbalanced group. Stub as no-op macros for both forms.
  def_macro_noop("\\internallinenumbers OptionalMatch:* [Number]")?;
  def_macro_noop("\\endinternallinenumbers")?;
  DefMacro!("\\csname internallinenumbers*\\endcsname OptionalMatch:* [Number]", "");
  DefMacro!("\\csname endinternallinenumbers*\\endcsname", "");

  def_macro_noop("\\linenumbers OptionalMatch:* [Number]")?;
  def_macro_noop("\\nolinenumbers")?;
  def_macro_noop("\\runninglinenumbers OptionalMatch:* [Number]")?;
  def_macro_noop("\\pagewiselinenumbers")?;
  def_macro_noop("\\realpagewiselinenumbers")?;
  def_macro_noop("\\runningpagewiselinenumbers")?;

  def_macro_noop("\\leftlinenumbers  OptionalMatch:*")?;
  def_macro_noop("\\rightlinenumbers OptionalMatch:*")?;
  def_macro_noop("\\switchlinenumbers OptionalMatch:*")?;

  def_macro_noop("\\setrunninglinenumbers")?;
  def_macro_noop("\\setpagewiselinenumbers")?;

  def_macro_noop("\\resetlinenumber OptionalMatch:* [Number]")?;
  def_macro_noop("\\modulolinenumbers [Number]")?;

  def_macro_noop("\\linenumberfont")?;
  DefRegister!("\\linenumbersep", Number(0));
  DefRegister!("\\linenumberwidth", Dimension(655360)); // 10pt

  def_macro_noop("\\thelinenumber")?;
  DefRegister!("\\c@linenumber", Number(0));
  DefRegister!("\\c@runninglinenumber", Number(0));
  DefRegister!("\\c@internallinenumber", Number(0));
  DefRegister!("\\c@internallinenumbers", Number(0));

  def_macro_noop("\\makeLineNumber")?;
  def_macro_noop("\\makeLineNumberRunning")?;
  def_macro_noop("\\makeLineNumberOdd")?;
  def_macro_noop("\\makeLineNumberEven")?;
  def_macro_noop("\\makeLineNumberRight")?;
  def_macro_noop("\\makeLineNumberLeft")?;
  def_macro_noop("\\LineNumber")?;

  DefMacro!("\\numquote",        "\\quote");
  DefMacro!("\\endnumquote",     "\\endquote");
  DefMacro!("\\numquotation",    "\\quote");
  DefMacro!("\\endnumquotation", "\\endquote");

  def_macro_noop("\\quotelinenumberfont")?;
  DefRegister!("\\quotelinenumbersep", Number(0));

  // lineno.sty:1077 `\newif\ifLineNumbers \LineNumbersfalse`, :1934-1935
  // `\newif\ifoddNumberedPage`, `\newif\ifcolumnwiselinenumbers`. Classes test
  // the switch: minimalist.sty:144 `\LocallyStopLineNumbers` =
  // `\LNturnsONfalse\ifLineNumbers\LNturnsONtrue\fi\nolinenumbers`, reached
  // from homework.cls:128 `\@maketitle` (homework-demo-{cn,de,en,es,fr,jp}).
  DefConditional!("\\ifLineNumbers");
  DefConditional!("\\ifoddNumberedPage");
  DefConditional!("\\ifcolumnwiselinenumbers");
  // lineno.sty:1445 `\linelabel{key}` marks the current line for `\lineref`
  // (= `\ref` of the line number); the line number itself is layout, so the
  // pair is the kernel label/ref (lineno manual; Perl's binding lacks both).
  // \lineref, \linerefr, \linerefp take optional [*] and optional [offset] (lineno.sty:2804-2825).
  DefMacro!("\\linelabel Semiverbatim", "\\label{#1}");
  DefMacro!("\\lineref OptionalMatch:* [] Semiverbatim", "\\ref{#3}");
  DefMacro!("\\linerefr OptionalMatch:* [] Semiverbatim", "\\ref{#3}");
  DefMacro!("\\linerefp OptionalMatch:* [] Semiverbatim", "\\ref{#3}");
});
