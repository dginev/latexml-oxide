use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // lineno.sty:2628 reads its options with `\ProcessKeyvalOptions`, which marks them processed: `\@curroptions` stays as it was.
  key_options_processed()?;
  // Perl: lineno.sty.ltxml — stub (line numbering not meaningful for XML). Its DefEnvironments boxed the body in
  // restricted horizontal mode (a `$$` there no display, a `\section` inside a box); lineno.sty:1157-1165 makes the
  // environments macros: `\par` and the switch, ended by `\par\@endpetrue`. `{pagewiselinenumbers*}`,
  // which lineno does not define, is kept from Perl's binding, without its `[Number]`.
  RawTeX!(r"\@namedef{linenumbers*}{\par\linenumbers*}
\@namedef{runninglinenumbers*}{\par\runninglinenumbers*}
\@namedef{pagewiselinenumbers*}{\par\pagewiselinenumbers}
\def\endlinenumbers{\par\@endpetrue}
\let\endrunninglinenumbers\endlinenumbers
\let\endpagewiselinenumbers\endlinenumbers
\expandafter\let\csname endlinenumbers*\endcsname\endlinenumbers
\expandafter\let\csname endrunninglinenumbers*\endcsname\endlinenumbers
\expandafter\let\csname endpagewiselinenumbers*\endcsname\endlinenumbers
\let\endnolinenumbers\endlinenumbers");
  // lineno.sty:2881-2908 `bframe` — a framed block (frame presentational; ulineno), which begins
  // and ends with `\par`: its body is paragraphs of their own, as setspace's `{spacing}`.
  DefEnvironment!("{bframe}",                       "#body", mode => "internal_vertical",
    before_digest_end => { leave_horizontal()?; });
  DefRegister!("\\bframerule", Dimension(26214)); // lineno.sty:2914 \fboxrule = 0.4pt
  DefRegister!("\\bframesep",  Dimension(196608)); // lineno.sty:2917 \fboxsep = 3pt
  // lineno.sty:1219-1271: `{linenomath}`/`{linenomath*}` are plain macros — penalties and holding inserts for
  // the line numbers, then `\ignorespaces`; `\endlinenomath` `\global\@ignoretrue` — so a `$$` in them is display
  // math, in the mode around it (tex.web:21715). Perl's DefEnvironments box the body in restricted horizontal
  // mode, where `$$` is two empty inline formulas and the formula ran as text: "Script _ can only appear in math
  // mode" (2307.10980, 2301.10600). The two styles are lineno's (:1219-1250), their penalties under `\ifLineNumbers`,
  // which stays false (line numbers are not modelled); `\linenomath` is a macro calling one, so a template's
  // `\ifx\linenomath\linenomathWithnumbers` (eccv.sty) takes the no-numbers branch, as with lineno's default
  // `\nolinenumberdisplaymath` (:1254-1263, :1280). `\linenopenalty`, `\linenopenaltypar`: lineno.sty:448, :467.
  RawTeX!(r"\newcount\linenopenalty\linenopenalty=-100000
\mathchardef\linenopenaltypar=32000
\def\@LN@outer@holdins{0}
\newcommand\linenomathNonumbers{\ifLineNumbers\ifnum\interlinepenalty>-\linenopenaltypar
\global\holdinginserts\thr@@\advance\interlinepenalty\linenopenalty
\ifhmode\advance\predisplaypenalty\linenopenalty\fi\fi\fi\ignorespaces}
\newcommand\linenomathWithnumbers{\ifLineNumbers\ifnum\interlinepenalty>-\linenopenaltypar
\global\holdinginserts\thr@@\advance\interlinepenalty\linenopenalty
\ifhmode\advance\predisplaypenalty\linenopenalty\fi
\advance\postdisplaypenalty\linenopenalty\advance\interdisplaylinepenalty\linenopenalty\fi\fi\ignorespaces}
\newcommand\linenumberdisplaymath{\def\linenomath{\linenomathWithnumbers}\@namedef{linenomath*}{\linenomathNonumbers}}
\newcommand\nolinenumberdisplaymath{\def\linenomath{\linenomathNonumbers}\@namedef{linenomath*}{\linenomathWithnumbers}}
\nolinenumberdisplaymath
\def\endlinenomath{\ifLineNumbers\global\holdinginserts\@LN@outer@holdins\fi\global\@ignoretrue}
\expandafter\let\csname endlinenomath*\endcsname\endlinenomath");

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
  // lineno.sty:2214 `\newcommand*\firstlinenumber[1]{\chardef\c@firstlinenumber#1\relax …}`
  // — the binding replaces the raw file, so the command must exist here
  // (lineno/lineno manual: `\firstlinenumber{1}`). Guard:
  // `perfect_kernel_batch56::sweep47_single_name_gaps`.
  DefMacro!("\\firstlinenumber{}", "\\chardef\\c@firstlinenumber#1\\relax");
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
  // lineno.sty:2151-2158 `\modulolinenumbers` takes a star (`\@ifstar`), then `[1][\z@]` (:2182);
  // Perl's `[Number]` (lineno.sty.ltxml:43) typeset the star and the option.
  def_macro_noop("\\modulolinenumbers OptionalMatch:* [Number]")?;

  def_macro_noop("\\linenumberfont")?;
  // lineno.sty:1549-1552 `\newdimen\linenumbersep \linenumbersep=10pt` (Perl: `Number(0)`).
  DefRegister!("\\linenumbersep", Dimension(655360)); // 10pt
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

  // lineno.sty:2849-2867: the quote environments number their lines through `\numquotelist`, whose
  // `\quotelinenumbers` reads a star or `[n]` as `\linenumbers` does; `{numquotation}` is a
  // `\quotation`, and the starred environments exist. Perl's `\numquote`/`\numquotation` = `\quote`
  // (lineno.sty.ltxml:58-61) typeset a `[n]` and left `{numquote*}` undefined. `\numquotelist` keeps
  // only `\quotelinenumbers`: lineno.sty:2855-2862 also sets `\leftlinenumbers`, `\linenumbersep` and
  // the number font, presentational here.
  RawTeX!(r"\newcommand\quotelinenumbers{\@ifstar\linenumbers{\@ifnextchar[\linenumbers{\linenumbers*}}}
\newcommand\numquotelist{\quotelinenumbers}
\newenvironment{numquote}{\quote\numquotelist}{\endquote}
\newenvironment{numquotation}{\quotation\numquotelist}{\endquotation}
\newenvironment{numquote*}{\quote\numquotelist*}{\endquote}
\newenvironment{numquotation*}{\quotation\numquotelist*}{\endquotation}");

  def_macro_noop("\\quotelinenumberfont")?;
  // lineno.sty:2852-2853 `\newdimen\quotelinenumbersep \quotelinenumbersep=\linenumbersep`.
  DefRegister!("\\quotelinenumbersep", Dimension(655360)); // 10pt

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
